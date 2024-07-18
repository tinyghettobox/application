use sea_orm::entity::prelude::*;
use sea_orm::Iterable;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use crate::util::{ChangeTracking, with_change_tracking};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "spotify_config")]
#[serde(rename = "SpotifyConfig")]
#[ts(export)]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip)]
    pub id: i32,
    pub client_id: String,
    pub secret_key: String,
    #[ts(optional)]
    pub refresh_token: Option<String>,
    #[ts(optional)]
    pub access_token: Option<String>,
    #[ts(optional)]
    pub expired_at: Option<String>,
    #[ts(optional)]
    pub username: Option<String>,
    #[ts(optional)]
    pub password: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

with_change_tracking!(ActiveModel);
