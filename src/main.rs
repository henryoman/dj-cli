use color_eyre::Result;
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use crossterm::cursor::{Hide, Show};
use crossterm::event::{EnableMouseCapture, DisableMouseCapture};
use tracing::{info, error};
use tracing_subscriber;

pub mod app;
pub mod ui;

use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize error handling
    color_eyre::install()?;
    
    // Initialize logging (2025 best practice)
    tracing_subscriber::fmt::init();
    
    info!("Starting DJ CLI");
    
    // Enable raw mode and mouse capture
    enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), Hide, EnableMouseCapture)?;
    
    // Initialize terminal
    let terminal = ratatui::init();
    
    // Run the app
    let app_result = App::new().run(terminal).await;
    
    // Restore terminal
    ratatui::restore();
    
    // Disable raw mode and mouse capture
    crossterm::execute!(std::io::stdout(), Show, DisableMouseCapture)?;
    disable_raw_mode()?;
    
    if let Err(e) = &app_result {
        error!("Application error: {}", e);
    }
    
    app_result
}
