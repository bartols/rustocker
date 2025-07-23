use crate::app::App;
use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn render_volumes(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    if app.volumes.is_empty() {
        let paragraph = Paragraph::new("No volumes found or loading...")
            .block(Block::default().title("Volumes").borders(Borders::ALL))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(paragraph, area);
    } else {
        let items: Vec<ListItem> = app
            .volumes
            .iter()
            .map(|volume| ListItem::new(volume.clone()).style(Style::default().fg(Color::White)))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!("Volumes ({})", app.volumes.len()))
                    .borders(Borders::ALL),
            )
            .style(Style::default());

        f.render_widget(list, area);
    }
}

pub fn render_volumes_help() -> &'static str {
    "[←/→] Switch Tab   [R/F5] Refresh   [C] Create Volume   [D] Delete   [Q/Esc/Ctrl+C] Quit"
}

// Future enhancement: render with detailed volume information
#[allow(dead_code)]
pub fn render_volumes_detailed(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    use ratatui::{
        layout::{Constraint, Direction, Layout},
        text::{Line, Span},
    };

    if app.volumes.is_empty() {
        let paragraph = Paragraph::new("No volumes found or loading...")
            .block(Block::default().title("Volumes").borders(Borders::ALL))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(paragraph, area);
        return;
    }

    // Split area for volume list and details
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Volume list (left side)
    let items: Vec<ListItem> = app
        .volumes
        .iter()
        .enumerate()
        .map(|(i, volume)| {
            let style = if i == 0 {
                // Highlight first volume (selected)
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(volume.clone()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!("Volumes ({})", app.volumes.len()))
                .borders(Borders::ALL),
        )
        .style(Style::default());

    f.render_widget(list, chunks[0]);

    // Volume details (right side)
    let details = if let Some(selected_volume) = app.volumes.first() {
        vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Blue)),
                Span::raw(selected_volume),
            ]),
            Line::from(vec![
                Span::styled("Driver: ", Style::default().fg(Color::Blue)),
                Span::raw("local"),
            ]),
            Line::from(vec![
                Span::styled("Mountpoint: ", Style::default().fg(Color::Blue)),
                Span::raw("/var/lib/docker/volumes/..."),
            ]),
        ]
    } else {
        vec![Line::from("No volume selected")]
    };

    let details_paragraph = Paragraph::new(details)
        .block(
            Block::default()
                .title("Volume Details")
                .borders(Borders::ALL),
        )
        .style(Style::default());

    f.render_widget(details_paragraph, chunks[1]);
}
