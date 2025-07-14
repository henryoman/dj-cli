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
    /// Batch mode enabled
    pub batch_mode: bool,
    /// List of URLs for batch download
    pub batch_urls: Vec<String>,
    /// Current batch download progress
    pub batch_progress: BatchProgress,
}

#[derive(Debug, Clone)]
pub struct BatchProgress {
    pub current: usize,
    pub total: usize,
    pub completed: Vec<String>,
    pub failed: Vec<String>,
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
            batch_mode: false,
            batch_urls: Vec::new(),
            batch_progress: BatchProgress {
                current: 0,
                total: 0,
                completed: Vec::new(),
                failed: Vec::new(),
            },
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
            KeyCode::Char('b') | KeyCode::Char('B') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Toggle batch mode with Ctrl+B
                    self.batch_mode = !self.batch_mode;
                    if self.batch_mode {
                        self.status_message = "🎯 Batch mode ON - Add URLs with Enter, download with Ctrl+D".to_string();
                        self.batch_urls.clear();
                        self.batch_progress = BatchProgress {
                            current: 0,
                            total: 0,
                            completed: Vec::new(),
                            failed: Vec::new(),
                        };
                    } else {
                        self.status_message = "Single URL mode - Paste a YouTube URL and press Enter to download".to_string();
                        self.batch_urls.clear();
                    }
                    info!("Batch mode toggled: {}", self.batch_mode);
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) && self.batch_mode {
                    // Start batch download with Ctrl+D
                    self.start_batch_download(128).await?;
                }
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
                if self.batch_mode {
                    // In batch mode, Enter adds URL to batch list
                    let url = self.input.trim().to_string();
                    if !url.is_empty() {
                        if url.contains("youtube.com") || url.contains("youtu.be") {
                            self.batch_urls.push(url.clone());
                            self.status_message = format!("✅ Added to batch: {} (Total: {})", url, self.batch_urls.len());
                            self.input.clear();
                        } else {
                            self.status_message = "❌ Please enter a valid YouTube URL".to_string();
                        }
                    }
                } else {
                    // Single URL mode - normal behavior
                    match self.focus {
                        Focus::Download256 => {
                            self.start_download(256).await?;
                        }
                        Focus::Download128 => {
                            self.start_download(128).await?;
                        }
                        Focus::Input => {
                            // Enter in input field triggers 128kbps download by default
                            self.start_download(128).await?;
                        }
                    }
                }
            }
            _ => {
                // Handle text input for the URL field (always active in batch mode, or when input is focused)
                if self.batch_mode || self.focus == Focus::Input {
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

    /// Start batch downloading multiple YouTube videos as MP3
    async fn start_batch_download(&mut self, bitrate: u32) -> Result<()> {
        if self.batch_urls.is_empty() {
            self.status_message = "❌ No URLs in batch. Add URLs with Enter first.".to_string();
            return Ok(());
        }

        info!("Starting batch download for {} URLs at {}kbps", self.batch_urls.len(), bitrate);
        self.download_status = DownloadStatus::Downloading;
        self.batch_progress = BatchProgress {
            current: 0,
            total: self.batch_urls.len(),
            completed: Vec::new(),
            failed: Vec::new(),
        };

        // Create output directory in user's Downloads folder
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let output_dir = PathBuf::from(home).join("Downloads").join("dj-cli");
        
        // Create directories if they don't exist
        tokio::fs::create_dir_all(&output_dir).await?;
        
        let libs_dir = output_dir.join("libs");
        tokio::fs::create_dir_all(&libs_dir).await?;

        // Download each URL in the batch
        for (index, url) in self.batch_urls.iter().enumerate() {
            self.batch_progress.current = index + 1;
            self.status_message = format!("📥 Downloading {}/{}: {}", index + 1, self.batch_urls.len(), url);

            match self.download_mp3(url.clone(), libs_dir.clone(), output_dir.clone(), bitrate).await {
                Ok(file_path) => {
                    info!("Successfully downloaded: {}", file_path);
                    self.batch_progress.completed.push(url.clone());
                }
                Err(e) => {
                    error!("Download failed for {}: {}", url, e);
                    self.batch_progress.failed.push(url.clone());
                }
            }
        }

        // Final status message
        let success_count = self.batch_progress.completed.len();
        let failed_count = self.batch_progress.failed.len();
        
        if failed_count == 0 {
            self.status_message = format!("✅ Batch complete! All {} downloads successful", success_count);
            self.download_status = DownloadStatus::Success(format!("{} files downloaded", success_count));
        } else {
            self.status_message = format!("⚠️ Batch complete: {} successful, {} failed", success_count, failed_count);
            self.download_status = DownloadStatus::Success(format!("{} successful, {} failed", success_count, failed_count));
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
