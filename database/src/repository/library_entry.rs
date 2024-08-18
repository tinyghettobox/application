use std::collections::HashSet;

use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, DbErr, EntityTrait, QueryFilter, QueryOrder, Statement, TransactionTrait};
use sea_orm::ActiveValue::Set;
use sea_orm::Order::Asc;
use serde_json::Value;

use crate::model::library_entry::{ActiveModel, Column, Entity, Model, ParentLink};
use crate::model::track_source::{Column as TrackSourceColumn, Entity as TrackSourceEntity};
use crate::repository::track_source::TrackSourceRepository;

pub struct LibraryEntryRepository {}

impl LibraryEntryRepository {
    pub async fn get(conn: &DatabaseConnection, id: i32) -> Result<Option<Model>, DbErr> {
        let mut entry = Entity::find_by_id(id).find_also_linked(ParentLink).one(conn).await?;

        if let Some((entry, parent)) = entry.as_mut() {
            entry.parent_name = parent.clone().map(|parent| parent.name);
            entry.children = Some(Self::get_children(conn, id).await?);
        }

        Ok(entry.map(|(entry, _)| entry))
    }

    pub async fn create(conn: &DatabaseConnection, parent_id: Option<i32>, entry: Value) -> Result<Model, DbErr> {
        let tx = conn.begin().await?;

        let id = Self::create_recursive(&tx, parent_id, entry).await?;

        tx.commit().await?;

        Self::get(conn, id).await?.ok_or(DbErr::RecordNotFound("No library entry created".to_string()))
    }

    pub async fn update(conn: &DatabaseConnection, id: i32, entry: Value) -> Result<Model, DbErr> {
        let tx = conn.begin().await?;

        let mut stack = vec![entry];
        while let Some(entry) = stack.pop() {
            if let None = entry.get("id") {
                return Err(DbErr::Custom("Update entry is missing id???".to_string()));
            }
            let entity_id = entry
                .get("id")
                .ok_or(DbErr::Custom("Entry is missing id???".to_string()))?
                .as_i64()
                .ok_or(DbErr::Custom("Entry id is not an integer".to_string()))? as i32;

            let mut model: ActiveModel = Entity::find_by_id(entity_id)
                .one(&tx)
                .await?
                .ok_or(DbErr::RecordNotFound(format!("No entry with id {} found", entity_id)))?
                .into();

            model.set_from_json(entry.clone())?;
            model.update(&tx).await?;

            if let Some(Value::Array(children)) = entry.get("children") {
                for child in children {
                    stack.push(child.clone());
                }
            }
        }

        let mut updated_model = Entity::find_by_id(id)
            .one(&tx)
            .await?
            .ok_or(DbErr::RecordNotFound("No library entry updated".to_string()))?;
        updated_model.children = Some(Self::get_children(&tx, id).await?);

        tx.commit().await?;

        Ok(updated_model)
    }

    pub async fn delete(conn: &DatabaseConnection, id: i32) -> Result<bool, DbErr> {
        let result = Entity::delete_by_id(id).exec(conn).await?;
        Ok(result.rows_affected > 0)
    }

    pub async fn get_tracks_in_parent(conn: &DatabaseConnection, library_entry_id: i32) -> Result<Vec<Model>, DbErr> {
        let library_entries = Entity::find().from_raw_sql(
            Statement::from_sql_and_values(DbBackend::Sqlite, r#"
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
            "#, [library_entry_id.into()])
        ).all(conn).await?;

        let entry_ids = library_entries.iter().map(|entry| entry.id).collect::<Vec<i32>>();
        let track_sources = TrackSourceEntity::find().filter(TrackSourceColumn::LibraryEntryId.is_in(entry_ids)).all(conn).await?;

        let parent_ids = library_entries.iter().filter_map(|entry| entry.parent_id).collect::<HashSet<_>>();
        let parent_entries = Entity::find().filter(Column::Id.is_in(parent_ids)).all(conn).await?;

        let entries_with_track_sources = library_entries
            .into_iter()
            .map(|mut entry| {
                entry.track_source = track_sources.iter().find(|source| source.library_entry_id == entry.id).cloned();
                if let Some(parent_id) = entry.parent_id.clone() {
                    if let Some(parent) = parent_entries.iter().find(|parent| parent.id == parent_id) {
                        entry.parent_name = Some(parent.name.to_string());
                        entry.parent_image = parent.image.clone();
                    }
                }
                entry
            })
            .collect::<Vec<Model>>();

        Ok(entries_with_track_sources)
    }

    async fn create_recursive<C: ConnectionTrait>(tx: &C, parent_id: Option<i32>, entry: Value) -> Result<i32, DbErr> {
        let mut created_model = None;
        let mut stack = match entry.as_array() {
            None => vec![(entry, parent_id)],
            Some(entries) => entries.iter().map(|entry| (entry.clone(), parent_id.clone())).collect::<Vec<(Value, Option<i32>)>>()
        };
        while let Some((entry, parent_id)) = stack.pop() {
            let mut model = ActiveModel::from_json(entry.clone())?;
            if let Some(parent_id) = parent_id {
                model.parent_id = Set(Some(parent_id));
            }
            let mut model = model.insert(tx).await?;

            if let Some(track_source) = entry.get("track_source") {
                model.track_source = Some(TrackSourceRepository::create(tx, model.id, track_source.clone()).await?);
            }

            if created_model.is_none() {
                created_model = Some(model.id);
            }

            if let Some(Value::Array(children)) = entry.get("children") {
                for child in children {
                    stack.push((child.clone(), Some(model.id)));
                }
            }
        }

        Ok(created_model.ok_or(DbErr::RecordNotInserted)?)
    }

    async fn get_children<C: ConnectionTrait>(conn: &C, id: i32) -> Result<Vec<Model>, DbErr> {
        Ok(Entity::find()
            .filter(Column::ParentId.eq(id))
            .order_by(Column::SortKey, Asc)
            .find_also_related(TrackSourceEntity)
            .all(conn)
            .await?
            .into_iter()
            .map(|(mut entry, track_source)| {
                entry.track_source = track_source;
                entry
            })
            .collect::<Vec<Model>>())
    }

    pub async fn set_played_at(conn: &DatabaseConnection, id: i32) -> Result<(), DbErr> {
        let model = Entity::find_by_id(id).one(conn).await?.ok_or(DbErr::RecordNotFound("No library entry found".to_string()))?;
        let mut active_model = ActiveModel::from(model);
        active_model.played_at = Set(Some(Utc::now()));
        active_model.update(conn).await?;
        Ok(())
    }
}
