use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{DefaultTerminal, Frame};
// Removed ratatui_input for simplicity
use regex::Regex;
use std::path::PathBuf;
use std::time::{Duration, UNIX_EPOCH};
use std::{fs, process::Stdio};
use tracing::{error, info, warn};

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
    /// Download history for display
    pub download_history: Vec<String>,
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
            status_message: "Paste a YouTube URL and press Enter to download MP3".to_string(),
            download_status: DownloadStatus::Idle,
            focus: Focus::Input,
            download_history: Vec::new(),
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

    /// Sanitize and validate input text
    fn sanitize_input(&mut self, input: &str) -> String {
        // First, truncate if too long
        let truncated = if input.len() > MAX_PASTE_LENGTH {
            warn!(
                "Input truncated from {} to {} characters",
                input.len(),
                MAX_PASTE_LENGTH
            );
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
            warn!(
                "Cleaned input truncated from {} to {} characters",
                cleaned.len(),
                MAX_INPUT_LENGTH
            );
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
            .split_whitespace() // Normalize whitespace
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Handle character input with proper sanitization
    fn handle_char_input(&mut self, c: char) {
        // Check if adding this character would exceed the limit
        if self.input.len() >= MAX_INPUT_LENGTH {
            warn!(
                "Input at maximum length ({}), ignoring character",
                MAX_INPUT_LENGTH
            );
            self.status_message = format!("Input limit reached ({MAX_INPUT_LENGTH} characters)");
            return;
        }

        // Filter out problematic characters
        if c.is_control() && c != '\t' {
            warn!("Ignoring control character: {:?}", c);
            return;
        }

        self.input.push(c);

        // Clear any previous status messages when user types normally
        if self.status_message.starts_with("Input limit reached")
            || self.status_message.starts_with("Large input sanitized")
        {
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
                    original_len,
                    sanitized.len()
                );
            } else {
                self.status_message = "Input cleaned and URL extracted".to_string();
            }
            info!(
                "Input sanitized: original {} chars → {} chars",
                original_len,
                sanitized.len()
            );
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
            self.status_message = format!("Error: {e}");
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
                    self.start_download(128).await?;
                }
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Delete => {
                self.input.clear();
            }
            KeyCode::Tab => {
                // Tab does nothing now since we only have input focus
                // Keeping this for compatibility but it doesn't change focus
            }
            KeyCode::F(5) => {
                // F5 to clear input and extract URL from current content
                if !self.input.is_empty() {
                    let original = self.input.clone();
                    self.handle_paste(&original);
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
                self.status_message =
                    "💡 Paste detected! Press F5 to clean and extract URL from pasted content"
                        .to_string();
                info!("Ctrl+V detected - user should use F5 for URL extraction");
            }
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Handle Ctrl+A - select all (just clear input for simplicity)
                info!("Ctrl+A detected - clearing input");
            }
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Ignore other Ctrl+char combinations
                } else {
                    // Handle regular character input
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
            self.status_message = format!("❌ Download failed: {e}");
        }

        Ok(())
    }

    /// Perform the actual download with proper error isolation
    async fn perform_download(&mut self, url: String, bitrate: u32) -> Result<()> {
        // Starting download silently
        self.download_status = DownloadStatus::Downloading;
        self.status_message = format!("🎵 Downloading MP3 at {bitrate}kbps... Please wait");

        // Clear the input field when download starts
        self.input.clear();

        // Download directly to Downloads folder (no subfolder)
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let output_dir = PathBuf::from(home).join("Downloads");

        // Download using yt-dlp - clean and simple
        let file_path = self
            .download_mp3(url, output_dir, bitrate)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Download failed: {}", e))?;
        // Download completed successfully
        self.download_status = DownloadStatus::Success(file_path.clone());
        self.status_message = format!("✅ Successfully downloaded: {file_path}");

        // Add to download history for display - extract just the filename
        if let Some(filename) = file_path.strip_prefix("✅ Downloaded: ") {
            self.download_history.push(filename.to_string());
        } else {
            // Fallback in case format changes
            self.download_history.push(file_path.clone());
        }

        Ok(())
    }

    /// Download MP3 using yt-dlp - clean and simple (2025 best practice)
    async fn download_mp3(
        &self,
        url: String,
        output_dir: PathBuf,
        bitrate: u32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Step 1: Get list of existing MP3 files BEFORE download
        let existing_mp3s = self.get_mp3_files(&output_dir).await.unwrap_or_default();

        // Step 2: Do the actual download (back to working logic)
        let output_template = output_dir.join("%(title)s.%(ext)s");

        let mut cmd = tokio::process::Command::new("yt-dlp");
        let bitrate_arg = format!("{bitrate}K");
        let output_arg = output_template.to_string_lossy().to_string();
        cmd.args([
            "--format",
            "bestaudio",       // Download ONLY audio stream (no video)
            "--extract-audio", // Extract to final format
            "--audio-format",
            "mp3", // Convert to MP3
            "--audio-quality",
            &bitrate_arg, // Bitrate (128K/256K)
            "--output",
            &output_arg,         // Save to Downloads/[title].mp3
            "--no-playlist",     // Single video only
            "--prefer-ffmpeg",   // Use ffmpeg for conversion
            "--embed-thumbnail", // Add album art
            "--add-metadata",    // Add metadata
            "--no-warnings",     // Suppress warnings
            "--quiet",           // Minimal output
            url.as_str(),        // YouTube URL
        ]);

        // Completely suppress all output to keep TUI clean
        cmd.stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null());

        let output = cmd
            .output()
            .await
            .map_err(|_| "yt-dlp not found. Please install: brew install yt-dlp".to_string())?;

        if !output.status.success() {
            return Err(
                "Download failed. Check if the YouTube URL is valid and accessible.".into(),
            );
        }

        // Give the file system a moment to update
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Step 3: Get list of MP3 files AFTER download
        let new_mp3s = self.get_mp3_files(&output_dir).await.unwrap_or_default();

        // Step 4: Find the NEW file (difference between before and after)
        let new_file = new_mp3s
            .iter()
            .find(|file| !existing_mp3s.contains(file))
            .cloned()
            .unwrap_or_else(|| {
                // If no new file found, try to get the most recently modified MP3
                fs::read_dir(&output_dir)
                    .ok()
                    .and_then(|entries| {
                        entries
                            .filter_map(|e| e.ok())
                            .filter(|e| e.path().extension().is_some_and(|ext| ext == "mp3"))
                            .max_by_key(|e| {
                                e.metadata()
                                    .and_then(|m| m.modified())
                                    .unwrap_or(UNIX_EPOCH)
                            })
                    })
                    .and_then(|e| e.file_name().into_string().ok())
                    .unwrap_or_else(|| "unknown.mp3".to_string())
            });

        Ok(format!("✅ Downloaded: {new_file}"))
    }

    /// Helper function to get all MP3 filenames in a directory
    async fn get_mp3_files(
        &self,
        dir: &PathBuf,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut mp3_files = Vec::new();

        // Check if directory exists
        if !dir.exists() {
            return Ok(mp3_files);
        }

        let mut entries = tokio::fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Only process files (not directories)
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "mp3" {
                        if let Some(filename) = path.file_name() {
                            if let Some(filename_str) = filename.to_str() {
                                // Only add non-empty filenames that actually contain text
                                if !filename_str.is_empty() && filename_str.len() > 4 {
                                    mp3_files.push(filename_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(mp3_files)
    }

    /// Get the current input value
    pub fn input_value(&self) -> &str {
        &self.input
    }

    /// Check if input field is focused
    pub fn is_input_focused(&self) -> bool {
        true // Always focused now since it's the only element
    }
}
