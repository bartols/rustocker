use crate::app::App;
use crate::theme::current_theme;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};

pub fn draw_ui(f: &mut Frame, app: &App) {
    let theme = current_theme();
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

    // Create tabs with theme
    let titles = app
        .components
        .iter()
        .map(|c| Line::from(vec![Span::raw(c.name())]))
        .collect::<Vec<_>>();

    let tabs = Tabs::new(titles)
        .select(app.active_tab)
        .block(
            Block::default()
                .title("Docker TUI")
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        )
        .style(theme.normal_style())
        .highlight_style(theme.selected_style());

    f.render_widget(tabs, chunks[0]);

    // Render main content based on active tab using UI modules
    if let Some(component) = app.components.iter().find(|c| c.tab() == app.active_tab) {
        component.render(f, chunks[1]);
    } else {
        let main = Paragraph::new("Unknown tab")
            .block(
                Block::default()
                    .title("Error")
                    .borders(Borders::ALL)
                    .border_style(theme.error_style()),
            )
            .style(theme.error_style());
        f.render_widget(main, chunks[1]);
    }

    // Help text - changes based on active tab using UI modules
    let help_text =
        if let Some(component) = app.components.iter().find(|c| c.tab() == app.active_tab) {
            component.render_help()
        } else {
            "[←/→] Switch Tab   [Q/Esc/Ctrl+C] Quit"
        };

    let help = Paragraph::new(help_text).style(theme.muted_style());
    f.render_widget(help, chunks[2]);
}
