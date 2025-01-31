use crate::error::Problem;
use actix_web::{get, put, web, HttpResponse, Responder, Result};
use database::{model::system_config::Model, DatabaseConnection, SystemConfigRepository};
use serde_json::json;
use tracing::{error, warn};

#[get("/api/system/config")]
pub async fn get(conn: web::Data<DatabaseConnection>) -> impl Responder {
    match SystemConfigRepository::get(&conn).await {
        Ok(Some(model)) => HttpResponse::Ok().json(model),
        Ok(None) => {
            warn!("System config not found");
            HttpResponse::NotFound().finish()
        }
        Err(error) => {
            error!("Failed to get system config: {:?}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[put("/api/system/config")]
pub async fn update(conn: web::Data<DatabaseConnection>, json: web::Json<serde_json::Value>) -> Result<impl Responder> {
    match SystemConfigRepository::update_from_json(&conn, json.into_inner()).await {
        Ok((updated_model, changed_fields)) => {
            run_update_commands(updated_model.clone(), changed_fields)?;
            Ok(HttpResponse::Ok().json(updated_model))
        }
        Err(error) => {
            warn!("Failed to set system config: {:?}", error);
            Ok(HttpResponse::BadRequest().json(json!({ "error": error.to_string() })))
        }
    }
}

fn run_update_commands(updated_model: Model, changed_fields: Vec<String>) -> Result<(), Problem> {
    if changed_fields.contains(&"hostname".to_string()) {
        crate::commands::set_hostname(updated_model.hostname.clone())?;
    }
    if changed_fields.contains(&"cpu_governor".to_string()) {
        crate::commands::set_cpu_governor(updated_model.cpu_governor.clone())?;
    }
    if changed_fields.contains(&"overclock_sd_card".to_string()) {
        crate::commands::set_overclock_sd_card(updated_model.overclock_sd_card.clone())?;
    }
    if changed_fields.contains(&"log_to_ram".to_string()) {
        crate::commands::set_log_to_ram(updated_model.log_to_ram.clone())?;
    }
    if changed_fields.contains(&"wait_for_network".to_string()) {
        crate::commands::set_wait_for_network(updated_model.wait_for_network.clone())?;
    }
    if changed_fields.contains(&"initial_turbo".to_string()) {
        crate::commands::set_initial_turbo(updated_model.initial_turbo.clone())?;
    }
    if changed_fields.contains(&"swap_enabled".to_string()) {
        crate::commands::set_swap_enabled(updated_model.swap_enabled.clone())?;
    }
    if changed_fields.contains(&"hdmi_rotate".to_string()) {
        crate::commands::set_hdmi_rotate(updated_model.hdmi_rotate.clone())?;
    }
    if changed_fields.contains(&"lcd_rotate".to_string()) {
        crate::commands::set_lcd_rotate(updated_model.lcd_rotate.clone())?;
    }
    if changed_fields.contains(&"display_brightness".to_string()) {
        crate::commands::set_display_brightness(updated_model.display_brightness.clone())?;
    }
    if changed_fields.contains(&"audio_device".to_string()) {
        crate::commands::set_audio_device(updated_model.audio_device.clone())?;
    }
    if changed_fields.contains(&"led_on_off_shim_pin".to_string()) {
        crate::commands::set_led_on_off_shim_pin(updated_model.led_on_off_shim_pin.clone())?;
    }
    Ok(())
}