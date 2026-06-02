use framebuffer::{Framebuffer, KdMode};

/// RGB-Farbstruktur für Rendering
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Erstellt eine neue Farbe aus einem String im Format "r,g,b"
    pub fn from_string(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid color format: {}. Expected r,g,b", s));
        }
        
        let r = parts[0].parse::<u8>().map_err(|e| format!("Invalid red component: {}", e))?;
        let g = parts[1].parse::<u8>().map_err(|e| format!("Invalid green component: {}", e))?;
        let b = parts[2].parse::<u8>().map_err(|e| format!("Invalid blue component: {}", e))?;
        
        Ok(Color { r, g, b })
    }
}

/// Initialisiert den Framebuffer
pub fn init_framebuffer(device: &str) -> Result<Framebuffer, String> {
    let framebuffer = Framebuffer::new(device)
        .map_err(|e| format!("Failed to open framebuffer: {}", e))?;
    
    // Wechsle in den Grafikmodus, wenn wir auf einer Konsole sind
    let _ = framebuffer.kd_mode(KdMode::Graphics);
    
    Ok(framebuffer)
}

/// Löscht den Framebuffer (setzt alles auf schwarz)
pub fn clear_framebuffer(framebuffer: &mut Framebuffer) {
    for i in 0..framebuffer.frame_size() {
        framebuffer.frame()[i] = 0;
    }
}

/// Zeichnet ein Bild auf den Framebuffer mit Zentrierung
pub fn draw_image_centered(
    framebuffer: &mut Framebuffer, 
    image: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
) {
    let fb_width = framebuffer.width() as u32;
    let fb_height = framebuffer.height() as u32;
    
    // Berechne Bildposition zur Zentrierung
    let x_offset = (fb_width - image.width()) / 2;
    let y_offset = (fb_height - image.height()) / 2;
    
    // Zeichne das Bild auf den Framebuffer
    for (x, y, pixel) in image.enumerate_pixels() {
        let fb_x = x + x_offset;
        let fb_y = y + y_offset;
        
        if fb_x < fb_width && fb_y < fb_height {
            let offset = (fb_y * framebuffer.line_length() as u32 + fb_x * framebuffer.bytes_per_pixel() as u32) as usize;
            if offset + 2 < framebuffer.frame_size() {
                framebuffer.frame()[offset] = pixel[2];     // B
                framebuffer.frame()[offset + 1] = pixel[1]; // G
                framebuffer.frame()[offset + 2] = pixel[0]; // R
            }
        }
    }
}

/// Zeichnet einen Fortschrittsbalken auf den Framebuffer
pub fn draw_progress_bar(
    framebuffer: &mut Framebuffer, 
    x: u32, 
    y: u32, 
    width: u32, 
    height: u32, 
    progress: u8, 
    color: &Color
) {
    let progress_width = (width as f32 * (progress as f32 / 100.0)) as u32;
    
    // Hintergrund (grau)
    for py in y..y+height {
        for px in x..x+width {
            let offset = (py * framebuffer.line_length() as u32 + px * framebuffer.bytes_per_pixel() as u32) as usize;
            if offset + 2 < framebuffer.frame_size() {
                framebuffer.frame()[offset] = 50;     // B
                framebuffer.frame()[offset + 1] = 50; // G
                framebuffer.frame()[offset + 2] = 50; // R
            }
        }
    }
    
    // Fortschritt (farbig)
    for py in y..y+height {
        for px in x..x+progress_width {
            let offset = (py * framebuffer.line_length() as u32 + px * framebuffer.bytes_per_pixel() as u32) as usize;
            if offset + 2 < framebuffer.frame_size() {
                framebuffer.frame()[offset] = color.b;     // B
                framebuffer.frame()[offset + 1] = color.g; // G
                framebuffer.frame()[offset + 2] = color.r; // R
            }
        }
    }
}