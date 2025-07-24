use color_eyre::Result;
use tracing::{info, error};
// use tracing_subscriber;

pub mod app;
pub mod ui;

use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize error handling
    color_eyre::install()?;
    
    // Disable logging to keep TUI clean
    // tracing_subscriber::fmt::init();
    
    info!("Starting DJ CLI");
    
    // Initialize terminal
    let terminal = ratatui::init();
    
    // Run the app
    let app_result = App::new().run(terminal).await;
    
    // Restore terminal
    ratatui::restore();
    
    if let Err(e) = &app_result {
        error!("Application error: {}", e);
    }
    
    app_result
}
