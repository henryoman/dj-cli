use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{DefaultTerminal, Frame};
// Removed ratatui_input for simplicity
use std::path::PathBuf;
use std::time::Duration;
use tracing::{info, warn, error};
use yt_dlp::Youtube;

/// Application state
#[derive(Debug)]
pub struct App {
    /// Should the application exit?
    pub running: bool,
    /// YouTube URL input
    pub input: String,
    /// Current status message
    pub status_message: String,
    /// Download status
    pub download_status: DownloadStatus,
    /// Focus state (Input or Convert button)
    pub focus: Focus,
}

#[derive(Debug, Clone)]
pub enum DownloadStatus {
    Idle,
    Downloading,
    Success(String),
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    Input,
    Download256,
    Download128,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            input: String::new(),
            status_message: "Paste a YouTube URL and press Convert to download MP3".to_string(),
            download_status: DownloadStatus::Idle,
            focus: Focus::Input,
        }
    }

    /// Main application loop
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        info!("Starting main app loop");
        
        while self.running {
            // Draw UI
            terminal.draw(|frame| self.draw(frame))?;
            
            // Handle events
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key).await?;
                }
            }
        }
        
        info!("App loop finished");
        Ok(())
    }

    /// Draw the application UI
    fn draw(&mut self, frame: &mut Frame) {
        crate::ui::render(frame, self);
    }

    /// Handle keyboard events
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        // Global quit command
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            info!("User quit with Ctrl+C");
            self.running = false;
            return Ok(());
        }

        match key.code {
            KeyCode::Esc => {
                info!("User quit with Escape");
                self.running = false;
            }
            KeyCode::Tab => {
                // Switch focus between input and buttons
                self.focus = match self.focus {
                    Focus::Input => Focus::Download256,
                    Focus::Download256 => Focus::Download128,
                    Focus::Download128 => Focus::Input,
                };
            }
            KeyCode::Enter => {
                match self.focus {
                    Focus::Download256 => {
                        self.start_download(256).await?;
                    }
                    Focus::Download128 => {
                        self.start_download(128).await?;
                    }
                    Focus::Input => {
                        // Enter in input field triggers 256kbps download by default
                        self.start_download(256).await?;
                    }
                }
            }
            _ => {
                // Only handle input if focused on input field
                if self.focus == Focus::Input {
                    // Handle text input for the URL field
                    match key.code {
                        KeyCode::Char(c) => {
                            self.input.push(c);
                        }
                        KeyCode::Backspace => {
                            self.input.pop();
                        }
                        _ => {}
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Start downloading the YouTube video as MP3
    async fn start_download(&mut self, bitrate: u32) -> Result<()> {
        let url = self.input.trim();
        
        if url.is_empty() {
            self.status_message = "Please enter a YouTube URL".to_string();
            warn!("Empty URL provided");
            return Ok(());
        }

        if !url.contains("youtube.com") && !url.contains("youtu.be") {
            self.status_message = "Please enter a valid YouTube URL".to_string();
            warn!("Invalid URL provided: {}", url);
            return Ok(());
        }

        info!("Starting download for URL: {} at {}kbps", url, bitrate);
        self.download_status = DownloadStatus::Downloading;
        self.status_message = format!("Downloading at {}kbps... Please wait", bitrate);

        // Create output directory in user's Downloads folder
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let output_dir = PathBuf::from(home).join("Downloads").join("dj-cli");
        
        // Create directories if they don't exist
        tokio::fs::create_dir_all(&output_dir).await?;
        
        let libs_dir = output_dir.join("libs");
        tokio::fs::create_dir_all(&libs_dir).await?;

                 // Download using yt-dlp
         match self.download_mp3(url.to_string(), libs_dir, output_dir, bitrate).await {
            Ok(file_path) => {
                info!("Successfully downloaded: {}", file_path);
                self.download_status = DownloadStatus::Success(file_path.clone());
                self.status_message = format!("✅ Successfully downloaded to: {}", file_path);
            }
            Err(e) => {
                error!("Download failed: {}", e);
                self.download_status = DownloadStatus::Error(e.to_string());
                self.status_message = format!("❌ Error: {}", e);
            }
        }

        Ok(())
    }

    /// Download MP3 using yt-dlp
    async fn download_mp3(
        &self,
        url: String,
        libs_dir: PathBuf,
        output_dir: PathBuf,
        bitrate: u32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Initialize yt-dlp with auto-download of dependencies
        let youtube = Youtube::with_new_binaries(libs_dir, output_dir.clone()).await?;

        // Download audio as MP3 with specified bitrate
        let filename = format!("audio_{}kbps.mp3", bitrate);
        let output_path = youtube.download_audio_stream_from_url(url, &filename).await?;
        
        Ok(output_path.to_string_lossy().to_string())
    }

    /// Get the current input value
    pub fn input_value(&self) -> &str {
        &self.input
    }

    /// Check if input field is focused
    pub fn is_input_focused(&self) -> bool {
        self.focus == Focus::Input
    }

    /// Check if 256kbps button is focused
    pub fn is_256_focused(&self) -> bool {
        self.focus == Focus::Download256
    }

    /// Check if 128kbps button is focused
    pub fn is_128_focused(&self) -> bool {
        self.focus == Focus::Download128
    }
}
