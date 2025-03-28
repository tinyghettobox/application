use crate::model::spotify_config::{ActiveModel, Entity as SpotifyConfig, Model};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait};
use crate::util::ChangeTracking;

pub struct SpotifyConfigRepository;

impl SpotifyConfigRepository {
    pub async fn get(conn: &DatabaseConnection) -> Result<Model, DbErr> {
        SpotifyConfig::find_by_id(1).one(conn).await.map(|model| {
            model.unwrap_or_else(|| Model {
                id: 1,
                client_id: "".to_string(),
                secret_key: "".to_string(),
                refresh_token: None,
                access_token: None,
                expired_at: None,
                username: None,
                password: None,
            })
        })
    }

    pub async fn update(conn: &DatabaseConnection, config: Model) -> Result<Model, DbErr> {
        let existing = SpotifyConfig::find_by_id(1).one(conn).await?;

        match existing {
            Some(prev_model) => {
                let mut model = ActiveModel::from(prev_model.clone());
                if prev_model.access_token != config.access_token {
                    model.access_token = Set(config.access_token);
                }
                if prev_model.refresh_token != config.refresh_token {
                    model.refresh_token = Set(config.refresh_token);
                }
                if prev_model.expired_at != config.expired_at {
                    model.expired_at = Set(config.expired_at);
                }
                if prev_model.username != config.username {
                    model.username = Set(config.username);
                }
                if prev_model.password != config.password {
                    model.password = Set(config.password);
                }

                model.update(conn).await
            }
            None => Err(DbErr::RecordNotFound("SpotifyConfig".to_string())),
        }
    }

    pub async fn update_from_json(conn: &DatabaseConnection, json: serde_json::Value) -> Result<(Model, Vec<String>), DbErr> {
        let existing = SpotifyConfig::find_by_id(1).one(conn).await?.ok_or(DbErr::RecordNotFound("SpotifyConfig".to_string()))?;

        let mut model = ActiveModel::from(existing);
        let changed_fields = model.update_from_json(json);
        let updated_model = model.update(conn).await?;

        Ok((updated_model, changed_fields))
    }
}
