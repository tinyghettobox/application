use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub device: String,

    #[arg(short, long, default_value_t = 0)]
    pub rotate: i8,

    #[arg(short, long, default_value_t = false)]
    pub keep_aspect_ratio: bool,

    #[arg(short, long)]
    pub image: String,
    
    #[arg(short, long)]
    pub message: Option<String>,
    
    #[arg(long, default_value_t = 0)]
    pub progress: u8,
    
    #[arg(long, default_value = "0,255,0")]
    pub progress_color: String,
    
    #[arg(long, default_value = "255,255,255")]
    pub text_color: String,
}
