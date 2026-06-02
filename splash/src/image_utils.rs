use std::path::Path;
use image::{GenericImageView, ImageBuffer, Rgba};

/// Dreht ein Bild um den angegebenen Winkel (0, 90, 180, 270 Grad)
pub fn rotate_image(
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>, 
    rotation: i8
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    if rotation == 0 {
        return img.clone();
    }
    
    let (width, height) = (img.width(), img.height());
    
    match rotation {
        90 => {
            let mut rotated = ImageBuffer::new(height, width);
            for (x, y, pixel) in img.enumerate_pixels() {
                rotated.put_pixel(height - y - 1, x, *pixel);
            }
            rotated
        },
        180 => {
            let mut rotated = ImageBuffer::new(width, height);
            for (x, y, pixel) in img.enumerate_pixels() {
                rotated.put_pixel(width - x - 1, height - y - 1, *pixel);
            }
            rotated
        },
        270 => {
            let mut rotated = ImageBuffer::new(height, width);
            for (x, y, pixel) in img.enumerate_pixels() {
                rotated.put_pixel(y, width - x - 1, *pixel);
            }
            rotated
        },
        _ => img.clone(),
    }
}

/// Skaliert ein Bild auf die angegebene Größe, mit der Option, das Seitenverhältnis beizubehalten
pub fn scale_image(
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>, 
    target_width: u32, 
    target_height: u32, 
    keep_aspect: bool
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    if !keep_aspect {
        return image::imageops::resize(
            img, 
            target_width, 
            target_height, 
            image::imageops::FilterType::Lanczos3
        );
    }
    
    let (width, height) = (img.width(), img.height());
    let target_ratio = target_width as f32 / target_height as f32;
    let img_ratio = width as f32 / height as f32;
    
    let (new_width, new_height) = if img_ratio > target_ratio {
        // Image is wider, scale to target width
        (target_width, (target_width as f32 / img_ratio) as u32)
    } else {
        // Image is taller, scale to target height
        ((target_height as f32 * img_ratio) as u32, target_height)
    };
    
    image::imageops::resize(
        img, 
        new_width, 
        new_height, 
        image::imageops::FilterType::Lanczos3
    )
}

/// Lädt ein Bild aus dem Dateisystem
pub fn load_image(path: &Path) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, String> {
    let img = image::open(path)
        .map_err(|e| format!("Failed to open image: {}", e))?
        .to_rgba8();
    Ok(img)
}