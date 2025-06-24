use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
// Removed ratatui_input for simplicity

use crate::app::{App, DownloadStatus};

/// Render the main UI
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(1), // Spacing
            Constraint::Length(3), // Input box
            Constraint::Length(1), // Spacing
            Constraint::Length(3), // Download buttons
            Constraint::Length(1), // Spacing  
            Constraint::Length(3), // Status
            Constraint::Min(0),    // Remaining space
        ])
        .split(area);

    // Title
    let title = Paragraph::new("🎵 DJ YouTube to MP3 Converter")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Input box
    let input_style = if app.is_input_focused() {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("YouTube URL")
        .border_style(if app.is_input_focused() {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        });

    let input_widget = Paragraph::new(app.input_value())
        .style(input_style)
        .block(input_block);
    frame.render_widget(input_widget, chunks[2]);

    // Download buttons area - split horizontally
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // 256kbps button
            Constraint::Percentage(50), // 128kbps button
        ])
        .split(chunks[4]);

    // 256kbps Download button
    let button_256_style = if app.is_256_focused() {
        Style::default()
            .bg(Color::Green)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    };

    let button_256_text = match app.download_status {
        DownloadStatus::Downloading => "⏳ Downloading...",
        _ => "🎵 Download at 256kbps",
    };

    let download_256_button = Paragraph::new(button_256_text)
        .style(button_256_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if app.is_256_focused() {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Gray)
                }),
        );
    frame.render_widget(download_256_button, button_chunks[0]);

    // 128kbps Download button  
    let button_128_style = if app.is_128_focused() {
        Style::default()
            .bg(Color::Blue)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD)
    };

    let button_128_text = match app.download_status {
        DownloadStatus::Downloading => "⏳ Downloading...",
        _ => "🎵 Download at 128kbps",
    };

    let download_128_button = Paragraph::new(button_128_text)
        .style(button_128_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if app.is_128_focused() {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Gray)
                }),
        );
    frame.render_widget(download_128_button, button_chunks[1]);

    // Status message
    let status_color = match app.download_status {
        DownloadStatus::Success(_) => Color::Green,
        DownloadStatus::Error(_) => Color::Red,
        DownloadStatus::Downloading => Color::Yellow,
        DownloadStatus::Idle => Color::Gray,
    };

    let status = Paragraph::new(app.status_message.as_str())
        .style(Style::default().fg(status_color))
        .block(Block::default().borders(Borders::ALL).title("Status"));
    frame.render_widget(status, chunks[6]);

    // Help text at the bottom
    let help_text = vec![
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Switch focus | "),
            Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Convert | "),
            Span::styled("Esc/Ctrl+C", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Quit"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default());
    
    if chunks.len() > 7 {
        frame.render_widget(help, chunks[7]);
    }
}


