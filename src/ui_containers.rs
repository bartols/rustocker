use crate::app::App;
use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn render_containers(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    if app.containers.is_empty() {
        // Show loading or empty state
        let paragraph = Paragraph::new("No containers found or loading...")
            .block(Block::default().title("Containers").borders(Borders::ALL))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(paragraph, area);
    } else {
        // Create list items from containers
        let items: Vec<ListItem> = app
            .containers
            .iter()
            .map(|container| {
                ListItem::new(container.clone()).style(Style::default().fg(Color::White))
            })
            .collect();

        // Create the list widget
        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!("Containers ({})", app.containers.len()))
                    .borders(Borders::ALL),
            )
            .style(Style::default());

        f.render_widget(list, area);
    }
}

pub fn render_containers_help() -> &'static str {
    "[←/→] Switch Tab   [R/F5] Refresh   [S] Start/Stop   [L] Logs   [Q/Esc/Ctrl+C] Quit"
}

// Future enhancement: render with detailed container information
#[allow(dead_code)]
pub fn render_containers_detailed(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    use ratatui::{
        layout::{Constraint, Direction, Layout},
        text::{Line, Span},
    };

    if app.containers.is_empty() {
        let paragraph = Paragraph::new("No containers found or loading...")
            .block(Block::default().title("Containers").borders(Borders::ALL))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(paragraph, area);
        return;
    }

    // Split area for container list and details
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Container list (left side)
    let items: Vec<ListItem> = app
        .containers
        .iter()
        .enumerate()
        .map(|(i, container)| {
            let style = if i == 0 {
                // Highlight first container (selected)
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(container.clone()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!("Containers ({})", app.containers.len()))
                .borders(Borders::ALL),
        )
        .style(Style::default());

    f.render_widget(list, chunks[0]);

    // Container details (right side)
    let details = if let Some(selected_container) = app.containers.first() {
        vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Blue)),
                Span::raw(selected_container),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Blue)),
                Span::styled("Running", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Image: ", Style::default().fg(Color::Blue)),
                Span::raw("nginx:latest"),
            ]),
        ]
    } else {
        vec![Line::from("No container selected")]
    };

    let details_paragraph = Paragraph::new(details)
        .block(
            Block::default()
                .title("Container Details")
                .borders(Borders::ALL),
        )
        .style(Style::default());

    f.render_widget(details_paragraph, chunks[1]);
}
