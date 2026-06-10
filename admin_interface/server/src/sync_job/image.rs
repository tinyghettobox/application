use std::io::{Cursor, Read};
use tracing::{debug, warn};

/// Downloads the image at `url`, resizes it to 180×180 JPEG, and returns the
/// compressed bytes. Returns an error string on failure (callers treat images
/// as optional and log / skip on error).
pub fn fetch_and_resize(url: &str) -> Result<Vec<u8>, String> {
    debug!(url, "fetching cover image");
    let response = ureq::get(url).call().map_err(|e| e.to_string())?;
    let mut bytes: Vec<u8> = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;

    let img = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
    let resized = img.resize_to_fill(180, 180, image::imageops::FilterType::Triangle);

    let mut out: Vec<u8> = Vec::new();
    resized
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Jpeg)
        .map_err(|e| e.to_string())?;
    Ok(out)
}
