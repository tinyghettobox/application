use clap::Parser;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use framebuffer::Framebuffer;

mod cli;
mod framebuffer;
mod image_utils;
mod renderer;
mod state;
mod systemd;

use crate::cli::Cli;
use crate::state::SplashState;

/// Aktualisiert den Bildschirm basierend auf dem aktuellen Zustand
fn update_screen(
    fb: &mut Framebuffer,
    state: &SplashState,
    progress_x: u32,
    progress_y: u32,
    progress_width: u32,
    progress_height: u32,
    progress_color: &framebuffer::Color,
    text_color: &framebuffer::Color
) {
    // Zeichne den Fortschrittsbalken
    framebuffer::draw_progress_bar(
        fb,
        progress_x,
        progress_y,
        progress_width,
        progress_height,
        state.progress,
        progress_color
    );
    
    // Zeichne den Text, falls vorhanden
    if let Some(message) = &state.message {
        renderer::render_text(
            fb,
            message,
            progress_x,
            progress_y + progress_height + 10,
            text_color,
            16.0
        );
    }
}

fn main() -> Result<(), String> {
    // Parse Kommandozeilenargumente
    let args = Cli::parse();
    
    // Parse Farben
    let progress_color = framebuffer::Color::from_string(&args.progress_color)?;
    let text_color = framebuffer::Color::from_string(&args.text_color)?;
    
    // Initialisiere den Framebuffer
    let mut fb = framebuffer::init_framebuffer(&args.device)?;
    
    // Lade und verarbeite das Bild
    let image_path = Path::new(&args.image);
    let mut img = image_utils::load_image(image_path)?;
    
    // Rotiere das Bild wenn nötig
    img = image_utils::rotate_image(&img, args.rotate);
    
    // Skaliere das Bild
    let scaled_img = image_utils::scale_image(
        &img, 
        fb.width() as u32, 
        fb.height() as u32, 
        args.keep_aspect_ratio
    );
    
    // Lösche den Framebuffer (schwarzer Hintergrund)
    framebuffer::clear_framebuffer(&mut fb);
    
    // Zeichne das Bild auf den Framebuffer
    framebuffer::draw_image_centered(&mut fb, &scaled_img);
    
    // Definiere Parameter für den Fortschrittsbalken
    let progress_height = 20;
    let progress_y = fb.height() as u32 - progress_height - 40; // 40 Pixel über dem unteren Rand für Text
    let progress_width = fb.width() as u32 * 3 / 4;
    let progress_x = (fb.width() as u32 - progress_width) / 2;
    
    // Initialisiere den Zustand mit den Kommandozeilenargumenten
    let splash_state = Arc::new(Mutex::new(SplashState::new(
        args.progress,
        args.message.clone()
    )));
    
    // Initialisiere den Bildschirm mit dem aktuellen Zustand
    if let Ok(initial_state) = splash_state.lock() {
        update_screen(
            &mut fb,
            &initial_state,
            progress_x,
            progress_y,
            progress_width,
            progress_height,
            &progress_color,
            &text_color
        );
    }
    
    // Erstelle FIFO und starte den Thread zum Lesen
    let fifo_path = state::create_fifo()?;
    let reader_state = splash_state.clone();
    let _fifo_thread = state::spawn_fifo_reader(fifo_path, reader_state);
    
    // Starte Thread zur Überwachung des systemd-Boot-Fortschritts
    let systemd_state = splash_state.clone();
    let _systemd_thread = systemd::spawn_systemd_monitor(systemd_state);
    
    // Haupt-Loop: Regelmäßig den Bildschirm aktualisieren
    loop {
        thread::sleep(std::time::Duration::from_millis(50)); // 20 FPS
        
        // Hole den aktuellen Zustand und aktualisiere den Bildschirm
        if let Ok(current_state) = splash_state.lock() {
            update_screen(
                &mut fb,
                &current_state,
                progress_x,
                progress_y,
                progress_width,
                progress_height,
                &progress_color,
                &text_color
            );
        }
    }
}
