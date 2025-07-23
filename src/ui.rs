use crate::app::App;
use crate::{ui_containers, ui_images, ui_networks, ui_volumes};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};

pub fn draw_ui(f: &mut Frame, app: &App) {
    let size = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Help text
        ])
        .split(size);

    // Create tabs
    let titles = ["Containers", "Images", "Networks", "Volumes"]
        .iter()
        .map(|t| Line::from(vec![Span::raw(*t)]))
        .collect::<Vec<_>>();

    let tabs = Tabs::new(titles)
        .select(app.active_tab)
        .block(Block::default().title("Docker TUI").borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow));

    f.render_widget(tabs, chunks[0]);

    // Render main content based on active tab
    match app.active_tab {
        0 => ui_containers::render_containers(f, chunks[1], app),
        1 => ui_images::render_images(f, chunks[1], app),
        2 => ui_networks::render_networks(f, chunks[1], app),
        3 => ui_volumes::render_volumes(f, chunks[1], app),
        _ => {
            let main = Paragraph::new("Unknown tab")
                .block(Block::default().title("Error").borders(Borders::ALL));
            f.render_widget(main, chunks[1]);
        }
    }

    // Help text - changes based on active tab
    let help_text = match app.active_tab {
        0 => ui_containers::render_containers_help(),
        1 => ui_images::render_images_help(),
        2 => ui_networks::render_networks_help(),
        3 => ui_volumes::render_volumes_help(),
        _ => "[←/→] Switch Tab   [Q/Esc/Ctrl+C] Quit",
    };

    let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}
