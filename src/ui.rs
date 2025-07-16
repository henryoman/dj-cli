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
            Constraint::Length(8), // Title (6 lines + 2 for borders)
            Constraint::Length(1), // Spacing
            Constraint::Length(3), // Input box
            Constraint::Length(1), // Spacing
            Constraint::Length(3), // Download buttons
            Constraint::Length(1), // Spacing  
            Constraint::Length(3), // Status
            Constraint::Length(if app.batch_mode { 6 } else { 0 }), // Batch URLs list (only in batch mode)
            Constraint::Length(4), // Instructions
            Constraint::Min(0),    // Remaining space
        ])
        .split(area);

    // Title
    let title_text = vec![
        Line::from(vec![
            Span::styled(" _____   ______          ___    _       _______ ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("(_____) (______)       _(___)_ (_)     (_______)", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("(_)  (_)     (_)      (_)   (_)(_)        (_)   ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("(_)  (_) _   (_)      (_)    _ (_)        (_)   ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("(_)__(_)( )__(_)      (_)___(_)(_)____  __(_)__ ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("(_____)  (____)         (___)  (______)(_______)", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
    ];
    
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(title, chunks[0]);

    // Input box
    let input_style = if app.batch_mode || app.is_input_focused() {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(if app.batch_mode { "YouTube URL" } else { "YouTube URL" })
        .border_style(if app.batch_mode || app.is_input_focused() {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        });

    let input_widget = Paragraph::new(app.input_value())
        .style(input_style)
        .block(input_block);
    frame.render_widget(input_widget, chunks[2]);

    // Download buttons area - split horizontally with narrower buttons
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(15), // 256kbps button (fixed width)
            Constraint::Length(3),  // Spacing
            Constraint::Length(15), // 128kbps button (fixed width)
            Constraint::Min(0),     // Remaining space
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
        _ => "🎵 256kbps",
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
        _ => "🎵 128kbps",
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
    frame.render_widget(download_128_button, button_chunks[2]);



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

    // Batch URLs list (only show in batch mode)
    if app.batch_mode && chunks.len() > 7 {
        let batch_text: Vec<Line> = if app.batch_urls.is_empty() {
            vec![Line::from(vec![
                Span::styled("No URLs added yet. Press Enter to add.", Style::default().fg(Color::Gray))
            ])]
        } else {
            let mut lines = vec![Line::from(vec![
                Span::styled(format!("Queue ({}):", app.batch_urls.len()), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            ])];
            
            for (i, url) in app.batch_urls.iter().enumerate() {
                let display_url = if url.len() > 30 {
                    format!("{}...", &url[..27])
                } else {
                    url.clone()
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::Yellow)),
                    Span::styled(display_url, Style::default().fg(Color::White))
                ]));
            }
            lines
        };

        let batch_widget = Paragraph::new(batch_text)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("Batch Queue"));
        frame.render_widget(batch_widget, chunks[7]);
    }

    // Instructions section
    let instructions_chunk = if app.batch_mode && chunks.len() > 9 { chunks[8] } else if chunks.len() > 8 { chunks[8] } else { chunks[chunks.len() - 2] };
    
    let instructions_text = if app.batch_mode {
        vec![
            Line::from(vec![
                Span::styled("📋 BATCH MODE:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            ]),
            Line::from(vec![
                Span::styled("1. ", Style::default().fg(Color::Yellow)),
                Span::raw("Paste URL, press "),
                Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" to add"),
            ]),
            Line::from(vec![
                Span::styled("2. ", Style::default().fg(Color::Yellow)),
                Span::raw("Press "),
                Span::styled("Ctrl+D", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" to start batch download"),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("📋 SINGLE MODE:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            ]),
            Line::from(vec![
                Span::styled("1. ", Style::default().fg(Color::Yellow)),
                Span::raw("Paste URL, press "),
                Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" to download"),
            ]),
            Line::from(vec![
                Span::styled("2. ", Style::default().fg(Color::Yellow)),
                Span::raw("Press "),
                Span::styled("Ctrl+B", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" for batch mode"),
            ]),
        ]
    };

    let instructions = Paragraph::new(instructions_text)
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("How to Use"));
    frame.render_widget(instructions, instructions_chunk);

    // Help text at the bottom
    let help_text = if app.batch_mode {
        vec![
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" - Add | "),
                Span::styled("Ctrl+D", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" - Download | "),
                Span::styled("Ctrl+B", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" - Toggle"),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("Tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" - Focus | "),
                Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" - Download | "),
                Span::styled("Ctrl+B", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" - Batch"),
            ]),
        ]
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default());
    
    let help_chunk = if app.batch_mode && chunks.len() > 9 { chunks[9] } else if chunks.len() > 8 { chunks[9] } else { chunks[chunks.len() - 1] };
    frame.render_widget(help, help_chunk);
}


