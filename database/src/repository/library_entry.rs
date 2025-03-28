use std::collections::HashSet;

use sea_orm::prelude::DateTimeUtc;
use sea_orm::ActiveValue::Set;
use sea_orm::Order::Asc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, DbErr,
    EntityTrait, QueryFilter, QueryOrder, QuerySelect, Statement, TransactionTrait,
};

use crate::model::library_entry::{ActiveModel, Column, CreateModel, Entity, Model, ParentLink};
use crate::model::track_source::{Column as TrackSourceColumn, Entity as TrackSourceEntity};
use crate::repository::track_source::TrackSourceRepository;

pub struct LibraryEntryRepository {}

impl LibraryEntryRepository {
    pub async fn get(conn: &DatabaseConnection, id: i32) -> Result<Option<Model>, DbErr> {
        let mut entry = Entity::find_by_id(id)
            .find_also_linked(ParentLink)
            .one(conn)
            .await?;

        if let Some((entry, parent)) = entry.as_mut() {
            entry.parent_name = parent.clone().map(|parent| parent.name);
            entry.children = Some(Self::get_children(conn, id).await?);
        }

        Ok(entry.map(|(entry, _)| entry))
    }

    pub async fn create(
        conn: &DatabaseConnection,
        parent_id: Option<i32>,
        entries: Vec<CreateModel>,
    ) -> Result<Vec<Model>, DbErr> {
        let tx = conn.begin().await?;

        let models = Self::create_recursive(&tx, parent_id, entries).await?;

        tx.commit().await?;

        Ok(models)
    }

    pub async fn update(conn: &DatabaseConnection, id: i32, entry: Model) -> Result<Model, DbErr> {
        let tx = conn.begin().await?;

        let mut stack = vec![entry];
        while let Some(entry) = stack.pop() {
            let mut model: ActiveModel = Entity::find_by_id(entry.id)
                .one(&tx)
                .await?
                .ok_or(DbErr::RecordNotFound(format!(
                    "No entry with id {} found",
                    entry.id
                )))?
                .into();

            model.update_from_model(entry.clone());
            model.update(&tx).await?;

            if let Some(children) = entry.children.as_ref() {
                for child in children {
                    stack.push(child.clone());
                }
            }
        }

        let mut updated_model =
            Entity::find_by_id(id)
                .one(&tx)
                .await?
                .ok_or(DbErr::RecordNotFound(
                    "No library entry updated".to_string(),
                ))?;
        updated_model.children = Some(Self::get_children(&tx, id).await?);

        tx.commit().await?;

        Ok(updated_model)
    }

    pub async fn delete(conn: &DatabaseConnection, id: i32) -> Result<bool, DbErr> {
        let result = Entity::delete_by_id(id).exec(conn).await?;
        Ok(result.rows_affected > 0)
    }

    pub async fn mark_played(
        conn: &DatabaseConnection,
        library_entry_id: i32,
        played_at: Option<DateTimeUtc>,
    ) -> Result<(), DbErr> {
        let mut model: ActiveModel = Entity::find_by_id(library_entry_id)
            .one(conn)
            .await?
            .ok_or(DbErr::RecordNotFound("No library entry found".to_string()))?
            .into();
        model.played_at = Set(played_at);

        model.update(conn).await?;
        Ok(())
    }

    pub async fn get_tracks_in_parent(
        conn: &DatabaseConnection,
        library_entry_id: i32,
    ) -> Result<Vec<Model>, DbErr> {
        let library_entries = Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                r#"
                WITH RECURSIVE library_hierarchy AS (
                    SELECT *, substr('0000' || sort_key, -4, 4) as path
                    FROM library_entry
                    WHERE parent_id = ?

                    UNION ALL

                    SELECT le.*, lh.path || '.' || substr('0000' || le.sort_key, -4, 4)
                    FROM library_entry le
                    INNER JOIN library_hierarchy lh ON le.parent_id = lh.id
                )
                SELECT * FROM library_hierarchy WHERE variant != 'folder' ORDER BY path ASC;
            "#,
                [library_entry_id.into()],
            ))
            .all(conn)
            .await?;

        let entry_ids = library_entries
            .iter()
            .map(|entry| entry.id)
            .collect::<Vec<i32>>();
        let track_sources = TrackSourceEntity::find()
            .filter(TrackSourceColumn::LibraryEntryId.is_in(entry_ids))
            // Skip file. Due to its size it heavily impacts query performance and isn't needed here
            .select_only()
            .columns(vec![
                TrackSourceColumn::Id,
                TrackSourceColumn::LibraryEntryId,
                TrackSourceColumn::Url,
                TrackSourceColumn::SpotifyId,
                TrackSourceColumn::SpotifyType,
                TrackSourceColumn::Title,
            ])
            .all(conn)
            .await?;

        let parent_ids = library_entries
            .iter()
            .filter_map(|entry| entry.parent_id)
            .collect::<HashSet<_>>();
        let parent_entries = Entity::find()
            .filter(Column::Id.is_in(parent_ids))
            .all(conn)
            .await?;

        let entries_with_track_sources = library_entries
            .into_iter()
            .map(|mut entry| {
                entry.track_source = track_sources
                    .iter()
                    .find(|source| source.library_entry_id.unwrap() == entry.id)
                    .cloned();
                if let Some(parent_id) = entry.parent_id.clone() {
                    if let Some(parent) =
                        parent_entries.iter().find(|parent| parent_id == parent.id)
                    {
                        entry.parent_name = Some(parent.name.to_string());
                        entry.parent_image = parent.image.clone();
                    }
                }
                entry
            })
            .collect::<Vec<Model>>();

        Ok(entries_with_track_sources)
    }

    async fn create_recursive<C: ConnectionTrait>(
        tx: &C,
        parent_id: Option<i32>,
        entries: Vec<CreateModel>,
    ) -> Result<Vec<Model>, DbErr> {
        let mut created_model_ids = vec![];
        let mut stack = entries
            .iter()
            .map(|entry| (entry.clone(), parent_id.clone(), 0))
            .collect::<Vec<(CreateModel, Option<i32>, i32)>>();
        while let Some((entry, parent_id, level)) = stack.pop() {
            let mut model = entry.to_active_model();
            if let Some(parent_id) = parent_id {
                model.parent_id = Set(Some(parent_id));
            }
            let mut model = model.insert(tx).await?;

            if let Some(track_source) = entry.track_source.as_ref() {
                model.track_source = Some(
                    // File track sources are created during upload without library_entry_id to
                    // keep memory footprint low and prevent having all files in memory at once
                    if let Some(track_source_id) = track_source.id.clone() {
                        TrackSourceRepository::set_library_entry_id(
                            tx,
                            track_source_id.to_owned(),
                            model.id,
                        )
                        .await?
                    } else {
                        TrackSourceRepository::create(tx, Some(model.id), track_source.clone())
                            .await?
                    },
                );
            }

            if level == 0 {
                created_model_ids.push(model.id);
            }

            if let Some(children) = entry.children.as_ref() {
                for child in children {
                    stack.push((child.clone(), Some(model.id), level + 1));
                }
            }
        }

        Ok(Entity::find()
            .filter(Column::Id.is_in(created_model_ids))
            .find_also_related(TrackSourceEntity)
            .all(tx)
            .await?
            .into_iter()
            .map(|(mut model, track_source)| {
                model.track_source = track_source;
                model
            })
            .collect::<Vec<Model>>())
    }

    async fn get_children<C: ConnectionTrait>(conn: &C, id: i32) -> Result<Vec<Model>, DbErr> {
        let entries = Entity::find()
            .filter(Column::ParentId.eq(id))
            .order_by(Column::SortKey, Asc)
            .all(conn)
            .await?;
        let entry_ids = entries.iter().map(|e| e.id).collect::<Vec<i32>>();
        let track_sources = TrackSourceEntity::find()
            .filter(TrackSourceColumn::LibraryEntryId.is_in(entry_ids))
            // Skip file. Due to its size it heavily impacts query performance and isn't needed here
            .select_only()
            .columns(vec![
                TrackSourceColumn::Id,
                TrackSourceColumn::LibraryEntryId,
                TrackSourceColumn::Url,
                TrackSourceColumn::SpotifyId,
                TrackSourceColumn::SpotifyType,
                TrackSourceColumn::Title,
            ])
            .all(conn)
            .await?;

        Ok(entries
            .into_iter()
            .map(|mut entry| {
                entry.track_source = track_sources.iter().find_map(|track| {
                    if track.library_entry_id.unwrap() == entry.id {
                        return Some(track.clone());
                    }
                    None
                });
                entry
            })
            .collect::<Vec<Model>>())
    }
}
