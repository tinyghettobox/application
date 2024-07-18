use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "track_source")]
#[serde(rename = "TrackSource")]
#[ts(export)]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    #[serde(skip_deserializing)]
    pub library_entry_id: i32,
    pub title: String,
    #[ts(optional)]
    pub url: Option<String>,
    #[serde(skip_serializing)]
    #[ts(optional)]
    pub file: Option<Vec<u8>>,
    #[ts(optional)]
    pub spotify_id: Option<String>,
    #[ts(optional)]
    pub spotify_type: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    LibraryEntry,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::LibraryEntry => Entity::belongs_to(super::library_entry::Entity)
                .from(Column::LibraryEntryId)
                .to(super::library_entry::Column::Id)
                .into(),
        }
    }
}

impl Related<super::library_entry::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LibraryEntry.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
