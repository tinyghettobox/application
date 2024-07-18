use crate::model::system_config::{ActiveModel, Entity as SystemConfig, Model};
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait};
use sea_orm::ActiveValue::Set;
use crate::util::ChangeTracking;

pub struct SystemConfigRepository;

impl SystemConfigRepository {
    pub async fn get(conn: &DatabaseConnection) -> Result<Option<Model>, DbErr> {
        SystemConfig::find_by_id(1).one(conn).await
    }

    pub async fn set(conn: &DatabaseConnection, json: serde_json::Value) -> Result<Model, DbErr> {
        let existing = SystemConfig::find_by_id(1).one(conn).await?;

        Ok(match existing {
            Some(prev_model) => {
                let mut model = ActiveModel::from(prev_model);
                model.set_from_json(json)?;
                model.update(conn).await?
            }
            None => {
                let model = ActiveModel::from_json(json)?;
                model.insert(conn).await?
            }
        })
    }

    pub async fn update_from_json(conn: &DatabaseConnection, json: serde_json::Value) -> Result<(Model, Vec<String>), DbErr> {
        let existing = SystemConfig::find_by_id(1).one(conn).await?.ok_or(DbErr::RecordNotFound("SystemConfig".to_string()))?;

        let mut model = ActiveModel::from(existing);
        let changed_fields = model.update_from_json(json);
        let updated_model = model.update(conn).await?;

        Ok((updated_model, changed_fields))
    }

    pub async fn get_volume(conn: &DatabaseConnection) -> Result<u8, DbErr> {
        let existing = Self::get(conn).await?;

        match existing {
            Some(model) => Ok(model.volume),
            None => Err(DbErr::RecordNotFound("SystemConfig".to_string()))
        }
    }

    pub async fn set_volume(conn: &DatabaseConnection, volume: u8) -> Result<(), DbErr> {
        let existing = Self::get(conn).await?;

        match existing {
            Some(prev_model) => {
                let mut model = ActiveModel::from(prev_model);
                model.volume = Set(volume);
                model.update(conn).await?;
                Ok(())
            }
            None => {
                Err(DbErr::RecordNotFound("SystemConfig".to_string()))
            }
        }
    }
}
