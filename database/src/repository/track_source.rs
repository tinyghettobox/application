use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ConnectionTrait, DbErr, EntityTrait, SelectColumns};
use serde_json::Value;

use crate::model::track_source;

pub struct TrackSourceRepository;

impl TrackSourceRepository {
    pub async fn get_file<C: ConnectionTrait>(conn: &C, id: i32) -> Result<Option<Vec<u8>>, DbErr> {
        let model = track_source::Entity::find_by_id(id)
            .select_column(track_source::Column::File)
            .one(conn)
            .await?;

        Ok(model.and_then(|m| m.file))
    }

    pub async fn create<C: ConnectionTrait>(
        conn: &C,
        library_entry_id: i32,
        entity: Value,
    ) -> Result<track_source::Model, DbErr> {
        let mut model = track_source::ActiveModel::from_json(entity)?;
        model.library_entry_id = Set(library_entry_id);
        let created_model = model.insert(conn).await?;

        Ok(created_model)
    }
}
