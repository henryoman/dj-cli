use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
// Removed ratatui_input for simplicity

use crate::app::{App, DownloadStatus};

/// Render the main UI
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Create main layout - dynamic constraints based on what needs to be shown
    let has_download_activity = matches!(app.download_status, DownloadStatus::Downloading) 
        || !app.download_history.is_empty();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Title (6 lines + 2 for borders)
            Constraint::Length(1), // Spacing
            Constraint::Length(3), // Input box
            Constraint::Length(1), // Spacing
            Constraint::Length(if app.batch_mode { 6 } else { 0 }), // Batch URLs list (only in batch mode)
            Constraint::Length(4), // Instructions
            Constraint::Length(2), // Tip
            Constraint::Length(if has_download_activity { 4 } else { 0 }), // Download status (conditional)
            Constraint::Min(0),    // Remaining space
        ])
        .split(area);

    // Title - Bright neon cyan (classic terminal green alternative)
    let title_text = vec![
        Line::from(vec![
            Span::styled(" _____   ______          ___    _       _______ ", Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("(_____) (______)       _(___)_ (_)     (_______)", Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("(_)  (_)     (_)      (_)   (_)(_)        (_)   ", Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("(_)  (_) _   (_)      (_)    _ (_)        (_)   ", Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("(_)__(_)( )__(_)      (_)___(_)(_)____  __(_)__ ", Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("(_____)  (____)         (___)  (______)(_______)", Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD)),
        ]),
    ];
    
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(title, chunks[0]);

    // Input box - Bright yellow when focused/batch mode
    let input_style = if app.batch_mode || app.is_input_focused() {
        Style::default().fg(Color::Rgb(255, 255, 0))  // Bright yellow
    } else {
        Style::default().fg(Color::Rgb(255, 255, 255))  // Bright white
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(if app.batch_mode { "YouTube URL" } else { "YouTube URL" })
        .border_style(if app.batch_mode || app.is_input_focused() {
            Style::default().fg(Color::Rgb(255, 255, 0))  // Bright yellow border
        } else {
            Style::default().fg(Color::Rgb(128, 128, 128))  // Gray
        });

    let input_widget = Paragraph::new(app.input_value())
        .style(input_style)
        .block(input_block);
    frame.render_widget(input_widget, chunks[2]);

    // Batch URLs list (only show in batch mode)
    if app.batch_mode && chunks.len() > 4 {
        let batch_text: Vec<Line> = if app.batch_urls.is_empty() {
            vec![Line::from(vec![
                Span::styled("No URLs added yet. Press Enter to add.", Style::default().fg(Color::Rgb(128, 128, 128)))  // Gray
            ])]
        } else {
            let mut lines = vec![Line::from(vec![
                Span::styled(format!("Queue ({}):", app.batch_urls.len()), Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD))  // Bright cyan
            ])];
            
            for (i, url) in app.batch_urls.iter().enumerate() {
                let display_url = if url.len() > 30 {
                    format!("{}...", &url[..27])
                } else {
                    url.clone()
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::Rgb(255, 255, 0))),  // Bright yellow numbers
                    Span::styled(display_url, Style::default().fg(Color::Rgb(255, 255, 255)))  // Bright white text
                ]));
            }
            lines
        };

        let batch_widget = Paragraph::new(batch_text)
            .style(Style::default().fg(Color::Rgb(255, 255, 255)))  // Bright white
            .block(Block::default().borders(Borders::ALL).title("Batch Queue"));
        frame.render_widget(batch_widget, chunks[4]);
    }

    // Instructions section
    let instructions_chunk = if app.batch_mode && chunks.len() > 5 { chunks[5] } else if chunks.len() > 4 { chunks[5] } else { chunks[chunks.len() - 2] };
    
    let instructions_text = if app.batch_mode {
        vec![
            Line::from(vec![
                Span::styled("📋 BATCH MODE:", Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD))  // Bright cyan
            ]),
            Line::from(vec![
                Span::styled("1. ", Style::default().fg(Color::Rgb(255, 255, 0))),  // Bright yellow
                Span::raw("Paste URL, press "),
                Span::styled("Enter", Style::default().fg(Color::Rgb(0, 255, 0)).add_modifier(Modifier::BOLD)),  // Bright green
                Span::raw(" to add"),
            ]),
            Line::from(vec![
                Span::styled("2. ", Style::default().fg(Color::Rgb(255, 255, 0))),  // Bright yellow
                Span::raw("Press "),
                Span::styled("Ctrl+D", Style::default().fg(Color::Rgb(0, 255, 0)).add_modifier(Modifier::BOLD)),  // Bright green
                Span::raw(" to start batch download"),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("📋 SINGLE MODE:", Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD))  // Bright cyan
            ]),
            Line::from(vec![
                Span::styled("1. ", Style::default().fg(Color::Rgb(255, 255, 0))),  // Bright yellow
                Span::raw("Paste URL, press "),
                Span::styled("Enter", Style::default().fg(Color::Rgb(0, 255, 0)).add_modifier(Modifier::BOLD)),  // Bright green
                Span::raw(" to download"),
            ]),
            Line::from(vec![
                Span::styled("2. ", Style::default().fg(Color::Rgb(255, 255, 0))),  // Bright yellow
                Span::raw("Press "),
                Span::styled("Ctrl+B", Style::default().fg(Color::Rgb(0, 255, 0)).add_modifier(Modifier::BOLD)),  // Bright green
                Span::raw(" for batch mode"),
            ]),
        ]
    };

    let instructions = Paragraph::new(instructions_text)
        .style(Style::default().fg(Color::Rgb(255, 255, 255)))  // Bright white
        .block(Block::default().borders(Borders::ALL).title("How to Use"));
    frame.render_widget(instructions, instructions_chunk);

    // Just show the tip at the bottom - much cleaner
    let tip_text = vec![
        Line::from(vec![
            Span::styled("💡 Tip:", Style::default().fg(Color::Rgb(0, 255, 255))),  // Bright cyan
            Span::raw(" Paste messy text - URLs auto-extracted, input sanitized & limited"),
        ]),
    ];

    let tip = Paragraph::new(tip_text)
        .style(Style::default().fg(Color::Rgb(200, 200, 200)))  // Light gray
        .block(Block::default());
    
    let tip_chunk = if app.batch_mode && chunks.len() > 6 { chunks[6] } else if chunks.len() > 5 { chunks[6] } else { chunks[chunks.len() - 3] };
    frame.render_widget(tip, tip_chunk);
    
    // Download status and history (only when there's activity)
    if has_download_activity {
        let mut status_lines = Vec::new();
        
        // Show current download status if downloading
        match &app.download_status {
            DownloadStatus::Downloading => {
                status_lines.push(Line::from(vec![
                    Span::styled("🎵 ", Style::default().fg(Color::Rgb(255, 255, 0))),
                    Span::styled("Downloading...", Style::default().fg(Color::Rgb(255, 255, 0)).add_modifier(Modifier::BOLD)),
                ]));
            }
            _ => {}
        }
        
        // Show recent downloads (last 2 to keep it compact)
        for download in app.download_history.iter().rev().take(2) {
            // Wrap long filenames - truncate at 50 chars and add ...
            let display_name = if download.len() > 50 {
                format!("{}...", &download[..47])
            } else {
                download.clone()
            };
            
            status_lines.push(Line::from(vec![
                Span::styled("✅ ", Style::default().fg(Color::Rgb(0, 255, 0))),
                Span::styled(display_name, Style::default().fg(Color::Rgb(0, 255, 0))),
            ]));
        }
        
        let status_widget = Paragraph::new(status_lines)
            .style(Style::default().fg(Color::Rgb(255, 255, 255)))
            .block(Block::default().borders(Borders::ALL).title("Downloads"))
            .wrap(Wrap { trim: true });
        
        let status_chunk = chunks[chunks.len() - 2];
        frame.render_widget(status_widget, status_chunk);
    }
}


