use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::OpenOptionsExt;
use std::thread;
use nix::sys::stat;
use nix::unistd;

/// Zustandsstruktur für die Splash-Anwendung
#[derive(Debug, Clone)]
pub struct SplashState {
    pub progress: u8,
    pub message: Option<String>,
}

impl SplashState {
    /// Erstellt einen neuen Zustand
    pub fn new(progress: u8, message: Option<String>) -> Self {
        SplashState { progress, message }
    }
}

/// Erstellt eine FIFO-Pipe für die externe Kommunikation
pub fn create_fifo() -> Result<PathBuf, String> {
    let fifo_path = PathBuf::from("/tmp/splash_fifo");
    
    // Lösche die FIFO falls sie bereits existiert
    if fifo_path.exists() {
        std::fs::remove_file(&fifo_path)
            .map_err(|e| format!("Failed to remove existing FIFO: {}", e))?;
    }
    
    // Erstelle eine neue FIFO mit den passenden Berechtigungen (666)
    unistd::mkfifo(&fifo_path, stat::Mode::S_IRWXU | stat::Mode::S_IRWXG | stat::Mode::S_IRWXO)
        .map_err(|e| format!("Failed to create FIFO: {}", e))?;
    
    Ok(fifo_path)
}

/// Startet einen Thread zum Lesen der FIFO und Aktualisieren des Zustands
pub fn spawn_fifo_reader(fifo_path: PathBuf, state: Arc<Mutex<SplashState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            // Öffne die FIFO zum Lesen
            match OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NONBLOCK)
                .open(&fifo_path) 
            {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    
                    // Lese Zeilen aus der FIFO
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            process_command(&line, &state);
                        }
                    }
                },
                Err(_) => {
                    // FIFO konnte nicht geöffnet werden, warte kurz und versuche es erneut
                    thread::sleep(std::time::Duration::from_millis(100));
                }
            }
            
            // Kurze Pause, damit die CPU nicht überlastet wird
            thread::sleep(std::time::Duration::from_millis(10));
        }
    })
}

/// Verarbeitet Befehle, die über die FIFO empfangen werden
fn process_command(command: &str, state: &Arc<Mutex<SplashState>>) {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    
    if parts.is_empty() {
        return;
    }
    
    // Atomare Aktualisierung des Zustands
    if let Ok(mut state) = state.lock() {
        match parts[0] {
            "PROGRESS" => {
                if parts.len() >= 2 {
                    if let Ok(progress) = parts[1].parse::<u8>() {
                        if progress <= 100 {
                            state.progress = progress;
                        }
                    }
                }
            },
            "MSG" => {
                if parts.len() >= 2 {
                    state.message = Some(parts[1..].join(" "));
                }
            },
            "QUIT" => {
                // Beende das Programm
                std::process::exit(0);
            },
            _ => {}
        }
    }
}