use std::fmt::{Debug, Display, Formatter};
use super::track_source::Model as TrackSource;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, TS)]
#[sea_orm(rs_type = "String", db_type = "String(Some(9))")]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Variant {
    #[sea_orm(string_value = "folder")]
    Folder,
    #[sea_orm(string_value = "stream")]
    Stream,
    #[sea_orm(string_value = "file")]
    File,
    #[sea_orm(string_value = "spotify")]
    Spotify,
}

impl Display for Variant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Variant::Folder => "folder".to_string(),
            Variant::Stream => "stream".to_string(),
            Variant::File => "file".to_string(),
            Variant::Spotify => "spotify".to_string()
        };
        write!(f, "{}", str)
    }
}

#[derive(Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "library_entry")]
#[serde(rename = "LibraryEntry")]
#[ts(export)]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    #[ts(optional)]
    pub parent_id: Option<i32>,
    pub variant: Variant,
    pub name: String,
    #[ts(optional)]
    pub image: Option<Vec<u8>>,
    #[ts(type = "string", optional)]
    pub played_at: Option<DateTimeUtc>,
    pub sort_key: i32,
    #[sea_orm(ignore)]
    #[ts(optional)]
    pub children: Option<Vec<Model>>, // Just used to pass children from API to client
    #[sea_orm(ignore)]
    #[ts(optional)]
    pub track_source: Option<TrackSource>, // Just used to pass children from API to client
    // Only relevant for the user interface
    #[sea_orm(ignore)]
    #[ts(optional)]
    pub parent_name: Option<String>
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "Entity", from = "Column::ParentId", to = "Column::Id")]
    Parent,
    #[sea_orm(has_many = "super::track_source::Entity")]
    Children,
    #[sea_orm(has_one = "super::track_source::Entity")]
    TrackSource,
}

impl Debug for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model")
            .field("id", &self.id)
            .field("parent_id", &self.parent_id)
            .field("variant", &self.variant)
            .field("name", &self.name)
            .field("image", &FormatImage(self.image.as_ref()))
            .field("played_at", &self.played_at)
            .field("sort_key", &self.sort_key)
            .field("children", &self.children)
            .field("track_source", &self.track_source)
            .field("parent_name", &self.parent_name)
            .finish()
    }
}

impl Related<super::track_source::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TrackSource.def()
    }
}

impl Related<Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Children.def()
    }
}

pub struct ParentLink;
impl Linked for ParentLink {
    type FromEntity = Entity;
    type ToEntity = Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Parent.def()]
    }
}

impl ActiveModelBehavior for ActiveModel {}


struct FormatImage<'a>(pub Option<&'a Vec<u8>>);
impl<'a> Debug for FormatImage<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            None => write!(f, "None"),
            Some(buffer) => write!(f, "[u8; {}]", buffer.len())
        }
    }
}