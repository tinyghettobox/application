use actix_files::NamedFile;
use actix_web::error::{ErrorInternalServerError, ErrorNotFound};
use actix_web::{get, web, Error, Result};
use std::env::current_dir;
use std::path::PathBuf;

fn get_out_dir() -> Result<PathBuf, Error> {
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
