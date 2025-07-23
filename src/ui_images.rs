use crate::app::App;
use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn render_images(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    if app.images.is_empty() {
        let paragraph = Paragraph::new("No images found or loading...")
            .block(Block::default().title("Images").borders(Borders::ALL))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(paragraph, area);
    } else {
        let items: Vec<ListItem> = app
            .images
            .iter()
            .map(|image| ListItem::new(image.clone()).style(Style::default().fg(Color::White)))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!("Images ({})", app.images.len()))
                    .borders(Borders::ALL),
            )
            .style(Style::default());

        f.render_widget(list, area);
    }
}

pub fn render_images_help() -> &'static str {
    "[←/→] Switch Tab   [R/F5] Refresh   [D] Delete Image   [Q/Esc/Ctrl+C] Quit"
}

// Future enhancement: render with detailed image information
#[allow(dead_code)]
pub fn render_images_detailed(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    use ratatui::{
        layout::{Constraint, Direction, Layout},
        text::{Line, Span},
    };

    if app.images.is_empty() {
        let paragraph = Paragraph::new("No images found or loading...")
            .block(Block::default().title("Images").borders(Borders::ALL))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(paragraph, area);
        return;
    }

    // Split area for image list and details
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Image list (left side)
    let items: Vec<ListItem> = app
        .images
        .iter()
        .enumerate()
        .map(|(i, image)| {
            let style = if i == 0 {
                // Highlight first image (selected)
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(image.clone()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!("Images ({})", app.images.len()))
                .borders(Borders::ALL),
        )
        .style(Style::default());

    f.render_widget(list, chunks[0]);

    // Image details (right side)
    let details = if let Some(selected_image) = app.images.first() {
        vec![
            Line::from(vec![
                Span::styled("Repository: ", Style::default().fg(Color::Blue)),
                Span::raw(selected_image),
            ]),
            Line::from(vec![
                Span::styled("Tag: ", Style::default().fg(Color::Blue)),
                Span::raw("latest"),
            ]),
            Line::from(vec![
                Span::styled("Size: ", Style::default().fg(Color::Blue)),
                Span::raw("~500MB"),
            ]),
        ]
    } else {
        vec![Line::from("No image selected")]
    };

    let details_paragraph = Paragraph::new(details)
        .block(
            Block::default()
                .title("Image Details")
                .borders(Borders::ALL),
        )
        .style(Style::default());

    f.render_widget(details_paragraph, chunks[1]);
}
