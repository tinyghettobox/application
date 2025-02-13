use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::Iterable;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use ts_rs::TS;

#[derive(Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "track_source")]
#[serde(rename = "TrackSource")]
#[ts(export)]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    #[serde(skip)]
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

impl ActiveModel {
    pub fn update_from_model(&mut self, model: Model) {
        for column in Column::iter() {
            let old_value = self.get(column);
            let new_value = model.get(column);

            if &new_value != old_value.as_ref() {
                self.set(column, new_value);
            }
        }
    }
}

impl Debug for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model")
            .field("id", &self.id)
            .field("library_entry_id", &self.library_entry_id)
            .field("title", &self.title)
            .field("spotify_id", &self.spotify_id)
            .field("spotify_type", &self.spotify_type)
            .field("url", &self.url)
            .field("file", &FormatFile(self.file.as_ref()))
            .finish()
    }
}

struct FormatFile<'a>(pub Option<&'a Vec<u8>>);
impl<'a> Debug for FormatFile<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            None => write!(f, "None"),
            Some(buffer) => write!(f, "[u8; {}]", buffer.len()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateModel {
    pub title: String,
    pub url: Option<String>,
    pub file: Option<Vec<u8>>,
    pub spotify_id: Option<String>,
    pub spotify_type: Option<String>,
}

impl CreateModel {
    // Somehow the function is not recognized by rust as used, adding allow dead code
    #[allow(dead_code)]
    pub fn to_active_model(&self) -> ActiveModel {
        let mut model = ActiveModel::new();
        model.title = Set(self.title.clone());
        model.url = Set(self.url.clone());
        model.file = Set(self.file.clone());
        model.spotify_id = Set(self.spotify_id.clone());
        model.spotify_type = Set(self.spotify_type.clone());
        model
    }
}
