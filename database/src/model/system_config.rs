use crate::util::{with_change_tracking, ChangeTracking};
use sea_orm::entity::prelude::*;
use sea_orm::Iterable;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "system_config")]
#[serde(rename = "SystemConfig")]
#[ts(export)]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip)]
    pub id: i32,
    pub sleep_timer: i32,
    pub idle_shutdown_timer: i32,
    pub display_off_timer: i32,
    pub hostname: String,
    pub cpu_governor: String,
    pub overclock_sd_card: bool,
    pub log_to_ram: bool,
    pub wait_for_network: bool,
    pub initial_turbo: bool,
    pub swap_enabled: bool,
    pub hdmi_rotate: i32,
    pub lcd_rotate: i32,
    pub display_brightness: i32,
    pub display_resolution_x: i32,
    pub display_resolution_y: i32,
    pub audio_device: String,
    pub volume: u8,
    pub max_volume: u8,
    pub led_on_off_shim_pin: i32,
    pub led_brightness: i32,
    pub led_brightness_dimmed: i32,
    pub power_off_btn_delay: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

with_change_tracking!(ActiveModel);

