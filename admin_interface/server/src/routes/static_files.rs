use std::env::current_dir;
use std::path::{Path, PathBuf};

use actix_files::NamedFile;
use actix_web::{Error, get, Result, web};
use actix_web::error::{ErrorInternalServerError, ErrorNotFound};
use tracing::warn;

fn get_out_dir() -> Result<PathBuf, Error> {
    if let Ok(ui_path) = std::env::var("UI_PATH") {
        let static_files_path = Path::new(&ui_path);
        if static_files_path.exists() {
            return Ok(static_files_path.to_path_buf());
        } else {
            warn!("Supplied UI_PATH={} does not exist", ui_path);
        }
    }

    let dir = current_dir().map_err(|e| ErrorInternalServerError(e))?;
    let mut path = dir.as_path();
    loop {
        if path.join("admin_interface/web_ui").exists() {
            let out_dir = path.join("admin_interface/web_ui/dist");
            if !out_dir.exists() {
                return Err(ErrorInternalServerError(
                    "web_ui/out does not exist. Please run `yarn build` in the web_ui directory".to_string(),
                ));
            }
            return Ok(out_dir);
        }
        match path.parent() {
            None => {
                return Err(ErrorInternalServerError("Could not find web_ui directory".to_string()));
            }
            Some(parent) => {
                path = parent;
            }
        }
    }
}

#[get("/{filename:.*}")]
pub async fn get(filename: web::Path<String>) -> Result<NamedFile> {
    let out_dir = get_out_dir()?;

    let filename = match filename.as_str() {
        "" => "index".to_string(),
        filename => filename.to_string(),
    };

    let file_path = out_dir.join(filename);
    if file_path.exists() {
        return Ok(NamedFile::open(file_path)?);
    }

    let html_path = out_dir.join("index.html");
    if html_path.exists() {
        return Ok(NamedFile::open(html_path)?);
    }

    Err(ErrorNotFound("Not Found"))
}
