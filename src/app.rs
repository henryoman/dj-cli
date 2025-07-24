use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{DefaultTerminal, Frame};
// Removed ratatui_input for simplicity
use std::path::PathBuf;
use std::time::Duration;
use tracing::{info, warn, error};
use std::process::Stdio;
use regex::Regex;

// Maximum input length to prevent memory issues and UI corruption
const MAX_INPUT_LENGTH: usize = 500;
const MAX_PASTE_LENGTH: usize = 10000;

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
    /// Should clear the frame on next draw
    pub should_clear: bool,
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
            should_clear: false,
        }
    }

    /// Main application loop
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        info!("Starting main app loop");
        
        while self.running {
            // Only clear the terminal if needed (best practice)
            if self.should_clear {
                terminal.clear()?;
                self.should_clear = false;
            }
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

    /// Sanitize and validate input text
    fn sanitize_input(&mut self, input: &str) -> String {
        // First, truncate if too long
        let truncated = if input.len() > MAX_PASTE_LENGTH {
            warn!("Input truncated from {} to {} characters", input.len(), MAX_PASTE_LENGTH);
            &input[..MAX_PASTE_LENGTH]
        } else {
            input
        };

        // Try to extract YouTube URL from the text
        if let Some(url) = self.extract_youtube_url(truncated) {
            info!("Extracted YouTube URL: {}", url);
            return url;
        }

        // If no URL found, clean the text and apply length limit
        let cleaned = self.clean_text(truncated);
        if cleaned.len() > MAX_INPUT_LENGTH {
            warn!("Cleaned input truncated from {} to {} characters", cleaned.len(), MAX_INPUT_LENGTH);
            cleaned[..MAX_INPUT_LENGTH].to_string()
        } else {
            cleaned
        }
    }

    /// Extract YouTube URL from messy text
    fn extract_youtube_url(&self, text: &str) -> Option<String> {
        // YouTube URL patterns (in order of preference)
        let patterns = [
            // Full URLs with https
            r"https://(?:www\.)?youtube\.com/watch\?v=([a-zA-Z0-9_-]+)(?:[&\w=]*)?",
            r"https://youtu\.be/([a-zA-Z0-9_-]+)(?:\?[&\w=]*)?",
            // URLs without https
            r"(?:www\.)?youtube\.com/watch\?v=([a-zA-Z0-9_-]+)(?:[&\w=]*)?",
            r"youtu\.be/([a-zA-Z0-9_-]+)(?:\?[&\w=]*)?",
            // Just the watch part
            r"watch\?v=([a-zA-Z0-9_-]+)(?:[&\w=]*)?",
        ];

        for pattern in &patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(captures) = regex.captures(text) {
                    if let Some(video_id) = captures.get(1) {
                        let url = format!("https://www.youtube.com/watch?v={}", video_id.as_str());
                        info!("Extracted YouTube URL from pattern '{}': {}", pattern, url);
                        return Some(url);
                    }
                }
            }
        }

        None
    }

    /// Clean text by removing control characters and normalizing whitespace
    fn clean_text(&self, text: &str) -> String {
        text.chars()
            .filter(|c| {
                // Keep printable ASCII, basic Unicode, and essential whitespace
                c.is_ascii_graphic() || c.is_ascii_whitespace() || (*c as u32) > 127
            })
            .collect::<String>()
            .split_whitespace()  // Normalize whitespace
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Handle character input with proper sanitization
    fn handle_char_input(&mut self, c: char) {
        // Check if adding this character would exceed the limit
        if self.input.len() >= MAX_INPUT_LENGTH {
            warn!("Input at maximum length ({}), ignoring character", MAX_INPUT_LENGTH);
            self.status_message = format!("Input limit reached ({} characters)", MAX_INPUT_LENGTH);
            return;
        }

        // Filter out problematic characters
        if c.is_control() && c != '\t' {
            warn!("Ignoring control character: {:?}", c);
            return;
        }

        self.input.push(c);
        
        // Clear any previous status messages when user types normally
        if self.status_message.starts_with("Input limit reached") || 
           self.status_message.starts_with("Large input sanitized") {
            self.status_message.clear();
        }
    }

    /// Handle paste operation with sanitization
    fn handle_paste(&mut self, pasted_text: &str) {
        let original_len = pasted_text.len();
        let sanitized = self.sanitize_input(pasted_text);
        
        if sanitized != pasted_text {
            if original_len > MAX_PASTE_LENGTH {
                self.status_message = format!(
                    "Large input sanitized: {} → {} chars (extracted URL or cleaned text)", 
                    original_len, sanitized.len()
                );
            } else {
                self.status_message = "Input cleaned and URL extracted".to_string();
            }
            info!("Input sanitized: original {} chars → {} chars", original_len, sanitized.len());
        }

        // Replace the current input with sanitized content
        self.input = sanitized;
    }

    /// Handle keyboard events with improved error handling and input sanitization
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        // Global quit command
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            info!("User quit with Ctrl+C");
            self.running = false;
            return Ok(());
        }

        // Use a separate method for handling that can't crash the UI
        if let Err(e) = self.handle_key_event_safe(key).await {
            error!("Error handling key event: {}", e);
            self.status_message = format!("Error: {}", e);
            // Don't crash the UI - just show the error message
        }

        Ok(())
    }

    /// Safe key event handling that catches errors
    async fn handle_key_event_safe(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.running = false;
            }
            KeyCode::Enter => {
                if !self.input.trim().is_empty() {
                    if self.batch_mode {
                        self.batch_urls.push(self.input.clone());
                        self.status_message = format!("Added URL {} to batch (total: {})", 
                            self.batch_urls.len(), self.batch_urls.len());
                        self.input.clear();
                    } else {
                        self.start_download(128).await?;
                    }
                }
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Delete => {
                self.input.clear();
            }
            KeyCode::Tab => {
                // Switch focus between input and buttons
                self.focus = match self.focus {
                    Focus::Input => Focus::Download256,
                    Focus::Download256 => Focus::Download128,
                    Focus::Download128 => Focus::Input,
                };
            }
            KeyCode::F(5) => {
                // F5 to clear input and extract URL from current content
                if !self.input.is_empty() {
                    let original = self.input.clone();
                    self.handle_paste(&original);
                }
            }
            KeyCode::Char(c @ ('b' | 'B')) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Toggle batch mode with Ctrl+B
                    self.batch_mode = !self.batch_mode;
                    self.should_clear = true;
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
                        self.batch_progress = BatchProgress {
                            current: 0,
                            total: 0,
                            completed: Vec::new(),
                            failed: Vec::new(),
                        };
                    }
                } else {
                    // Handle regular 'b' character input
                    if self.batch_mode || self.focus == Focus::Input {
                        self.handle_char_input(c);
                    }
                }
            }
            KeyCode::Char(c @ ('d' | 'D')) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) && self.batch_mode {
                    // Start batch download with Ctrl+D
                    self.start_batch_download(128).await?;
                } else {
                    // Handle regular 'd' character input
                    if self.batch_mode || self.focus == Focus::Input {
                        self.handle_char_input(c);
                    }
                }
            }
            KeyCode::Char('1') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if !self.input.trim().is_empty() {
                    self.start_download(128).await?;
                }
            }
            KeyCode::Char('2') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if !self.input.trim().is_empty() {
                    self.start_download(256).await?;
                }
            }
            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Handle Ctrl+V paste - for now just inform user about F5
                self.status_message = "💡 Paste detected! Press F5 to clean and extract URL from pasted content".to_string();
                info!("Ctrl+V detected - user should use F5 for URL extraction");
            }
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Handle Ctrl+A - select all (just clear input for simplicity)
                info!("Ctrl+A detected - clearing input");
            }
            KeyCode::Char(c) => {
                // Handle other character input
                if self.batch_mode || self.focus == Focus::Input {
                    self.handle_char_input(c);
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Start downloading the YouTube video as MP3 with robust error handling
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

        // Wrap download in error handling to prevent crashes
        if let Err(e) = self.perform_download(url.to_string(), bitrate).await {
            error!("Download failed: {}", e);
            self.download_status = DownloadStatus::Error(e.to_string());
            self.status_message = format!("❌ Download failed: {}", e);
        }
        
        Ok(())
    }

    /// Perform the actual download with proper error isolation
    async fn perform_download(&mut self, url: String, bitrate: u32) -> Result<()> {

        // Starting download silently  
        self.download_status = DownloadStatus::Downloading;
        self.status_message = format!("🎵 Downloading MP3 at {}kbps... Please wait", bitrate);
        
        // Clear the input field when download starts
        self.input.clear();

        // Download directly to Downloads folder (no subfolder)
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let output_dir = PathBuf::from(home).join("Downloads");

        // Download using yt-dlp - clean and simple
        let file_path = self.download_mp3(url, output_dir, bitrate).await
            .map_err(|e| color_eyre::eyre::eyre!("Download failed: {}", e))?;
        // Download completed successfully
        self.download_status = DownloadStatus::Success(file_path.clone());
        self.status_message = format!("✅ Successfully downloaded: {}", file_path);

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

        // Download directly to Downloads folder (no subfolder)  
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let output_dir = PathBuf::from(home).join("Downloads");

        // Download each URL in the batch
        for (index, url) in self.batch_urls.iter().enumerate() {
            self.batch_progress.current = index + 1;
            self.status_message = format!("📥 Downloading {}/{}: {}", index + 1, self.batch_urls.len(), url);

            match self.download_mp3(url.clone(), output_dir.clone(), bitrate).await {
                Ok(_file_path) => {
                    // Batch download completed successfully
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

    /// Download MP3 using rusty_ytdl (pure Rust, no external dependencies)
    /// Download MP3 using yt-dlp - clean and simple (2025 best practice)
    async fn download_mp3(
        &self,
        url: String,
        output_dir: PathBuf,
        bitrate: u32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Create simple filename template - just title.mp3 in Downloads
        let output_template = output_dir.join("%(title)s.%(ext)s");
        
        let mut cmd = tokio::process::Command::new("yt-dlp");
        cmd.args(&[
            "--extract-audio",                              // Audio only - no video
            "--audio-format", "mp3",                        // MP3 format
            "--audio-quality", &format!("{}K", bitrate),    // Bitrate (128K/256K)
            "--output", &output_template.to_string_lossy(), // Save to Downloads/[title].mp3
            "--no-playlist",                                // Single video only
            "--prefer-ffmpeg",                             // Use ffmpeg
            "--embed-thumbnail",                           // Add album art
            "--add-metadata",                              // Add metadata
            "--no-warnings",                               // Suppress warnings
            "--quiet",                                     // Minimal output
            &url                                           // YouTube URL
        ]);

        // Completely suppress all output to keep TUI clean
        cmd.stdout(Stdio::null())
           .stderr(Stdio::null())
           .stdin(Stdio::null());
        
        let output = cmd.output().await
            .map_err(|_| format!("yt-dlp not found. Please install: brew install yt-dlp"))?;

        if !output.status.success() {
            return Err("Download failed. Check if the YouTube URL is valid and accessible.".into());
        }

        // Find the downloaded MP3 file in Downloads folder
        let mut files = tokio::fs::read_dir(&output_dir).await?;
        let mut newest_mp3: Option<(PathBuf, std::time::SystemTime)> = None;
        
        while let Some(file) = files.next_entry().await? {
            let path = file.path();
            if path.extension().and_then(|s| s.to_str()) == Some("mp3") {
                if let Ok(metadata) = file.metadata().await {
                    if let Ok(modified) = metadata.modified() {
                        match newest_mp3 {
                            None => newest_mp3 = Some((path, modified)),
                            Some((_, existing_time)) if modified > existing_time => {
                                newest_mp3 = Some((path, modified));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        match newest_mp3 {
            Some((path, _)) => {
                let filename = path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("audio.mp3");
                Ok(format!("✅ Downloaded: {}", filename))
            }
            None => Err("MP3 file not found after download".into())
        }
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
