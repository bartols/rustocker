use crate::app::App;
use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn render_networks(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    if app.networks.is_empty() {
        let paragraph = Paragraph::new("No networks found or loading...")
            .block(Block::default().title("Networks").borders(Borders::ALL))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(paragraph, area);
    } else {
        let items: Vec<ListItem> = app
            .networks
            .iter()
            .map(|network| ListItem::new(network.clone()).style(Style::default().fg(Color::White)))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!("Networks ({})", app.networks.len()))
                    .borders(Borders::ALL),
            )
            .style(Style::default());

        f.render_widget(list, area);
    }
}

pub fn render_networks_help() -> &'static str {
    "[←/→] Switch Tab   [R/F5] Refresh   [C] Create Network   [D] Delete   [Q/Esc/Ctrl+C] Quit"
}

// Future enhancement: render with detailed network information
#[allow(dead_code)]
pub fn render_networks_detailed(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    use ratatui::{
        layout::{Constraint, Direction, Layout},
        text::{Line, Span},
    };

    if app.networks.is_empty() {
        let paragraph = Paragraph::new("No networks found or loading...")
            .block(Block::default().title("Networks").borders(Borders::ALL))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(paragraph, area);
        return;
    }

    // Split area for network list and details
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Network list (left side)
    let items: Vec<ListItem> = app
        .networks
        .iter()
        .enumerate()
        .map(|(i, network)| {
            let style = if i == 0 {
                // Highlight first network (selected)
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(network.clone()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!("Networks ({})", app.networks.len()))
                .borders(Borders::ALL),
        )
        .style(Style::default());

    f.render_widget(list, chunks[0]);

    // Network details (right side)
    let details = if let Some(selected_network) = app.networks.first() {
        vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Blue)),
                Span::raw(selected_network),
            ]),
            Line::from(vec![
                Span::styled("Driver: ", Style::default().fg(Color::Blue)),
                Span::raw("bridge"),
            ]),
            Line::from(vec![
                Span::styled("Scope: ", Style::default().fg(Color::Blue)),
                Span::raw("local"),
            ]),
        ]
    } else {
        vec![Line::from("No network selected")]
    };

    let details_paragraph = Paragraph::new(details)
        .block(
            Block::default()
                .title("Network Details")
                .borders(Borders::ALL),
        )
        .style(Style::default());

    f.render_widget(details_paragraph, chunks[1]);
}
