use fontdue::{Font, FontSettings};
use framebuffer::Framebuffer;
use crate::framebuffer::Color;

/// Rendert Text auf den Framebuffer
pub fn render_text(
    framebuffer: &mut Framebuffer, 
    text: &str, 
    x: u32, 
    y: u32, 
    color: &Color,
    font_size: f32
) {
    // Lade die eingebettete Standardschriftart
    let font_data = include_bytes!("../../resources/DejaVuSans.ttf");
    let font = Font::from_bytes(font_data as &[u8], FontSettings::default())
        .expect("Failed to load font");

    let mut cursor_x = x;
    let scale = font_size;
    
    for c in text.chars() {
        let (metrics, bitmap) = font.rasterize(c, scale);
        
        for (i, alpha) in bitmap.iter().enumerate() {
            let bx = i % metrics.width;
            let by = i / metrics.width;
            
            let px = cursor_x + bx as u32;
            let py = y + by as u32;
            
            if px < framebuffer.width() as u32 && py < framebuffer.height() as u32 {
                let offset = (py * framebuffer.line_length() as u32 + px * framebuffer.bytes_per_pixel() as u32) as usize;
                if offset + 2 < framebuffer.frame_size() {
                    framebuffer.frame()[offset] = (color.b as u16 * *alpha as u16 / 255) as u8;     // B
                    framebuffer.frame()[offset + 1] = (color.g as u16 * *alpha as u16 / 255) as u8; // G
                    framebuffer.frame()[offset + 2] = (color.r as u16 * *alpha as u16 / 255) as u8; // R
                }
            }
        }
        
        cursor_x += metrics.width as u32 + 1; // +1 für Abstand zwischen Zeichen
    }
}