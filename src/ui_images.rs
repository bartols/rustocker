use crate::components::Component;
use crate::docker::{DockerClient, ImageInfo, ImageInspectDetails};

use async_trait::async_trait;
use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ImagesUI {
    tab_num: usize,
    docker_client: Arc<Mutex<DockerClient>>,
    selected_index: usize,
    images: Vec<ImageInfo>,
    last_tick: std::time::Instant,
    // Modal state
    show_inspect_modal: bool,
    inspect_data: Option<ImageInspectDetails>,
    inspect_scroll: usize,
}

impl ImagesUI {
    pub fn new(docker_client: Arc<Mutex<DockerClient>>, tab_num: usize) -> Self {
        Self {
            tab_num,
            docker_client,
            selected_index: 0,
            images: Vec::new(),
            last_tick: std::time::Instant::now(),
            show_inspect_modal: false,
            inspect_data: None,
            inspect_scroll: 0,
        }
    }

    async fn refresh_now(&mut self) -> Result<()> {
        let client = self.docker_client.lock().await;
        match client.list_images().await {
            Ok(images) => {
                self.images = images;
                // Adjust selected index if necessary
                if self.selected_index >= self.images.len() && !self.images.is_empty() {
                    self.selected_index = self.images.len() - 1;
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to refresh images: {}", e);
                Err(e.into())
            }
        }
    }

    fn get_selected_image(&self) -> Option<&ImageInfo> {
        self.images.get(self.selected_index)
    }

    async fn delete_image(&self, image: &ImageInfo) -> Result<()> {
        eprintln!("Deleting image: {}", image.repo_tag);
        // TODO: Implement image deletion using image.id
        // Should ask for confirmation and handle dependencies
        Ok(())
    }

    async fn pull_image(&self, image: &ImageInfo) -> Result<()> {
        if image.repo_tag == "<none>:<none>" {
            eprintln!("Cannot pull image without tag");
            return Ok(());
        }

        eprintln!("Pulling image: {}", image.repo_tag);
        // TODO: Implement image pull
        // Should show progress if possible
        Ok(())
    }

    async fn inspect_image(&mut self, image: &ImageInfo) -> Result<()> {
        // Show modal immediately with loading state
        self.show_inspect_modal = true;
        self.inspect_data = None;
        self.inspect_scroll = 0;

        // Fetch inspection data in background
        let client = self.docker_client.lock().await;
        match client.inspect_image(&image.id).await {
            Ok(details) => {
                self.inspect_data = Some(details);
            }
            Err(e) => {
                eprintln!("Failed to inspect image: {}", e);
                self.show_inspect_modal = false; // Close modal on error
            }
        }

        Ok(())
    }

    fn render_main_table(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        if self.images.is_empty() {
            let paragraph = Paragraph::new("No images found or loading...")
                .block(Block::default().title("Images").borders(Borders::ALL))
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(paragraph, area);
        } else {
            use ratatui::layout::Constraint;
            use ratatui::widgets::{Cell, Row, Table};

            // Create table headers
            let headers = Row::new(vec![
                Cell::from("Repository:Tag").style(Style::default().fg(Color::Yellow)),
                Cell::from("Image ID").style(Style::default().fg(Color::Yellow)),
                Cell::from("Size").style(Style::default().fg(Color::Yellow)),
                Cell::from("Created").style(Style::default().fg(Color::Yellow)),
                Cell::from("Containers").style(Style::default().fg(Color::Yellow)),
            ]);

            // Create table rows using pre-formatted data
            let rows: Vec<Row> = self
                .images
                .iter()
                .enumerate()
                .map(|(i, image)| {
                    let style = if i == self.selected_index {
                        Style::default().fg(Color::LightYellow).bg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    Row::new(vec![
                        Cell::from(image.repo_tag.clone()),
                        Cell::from(image.display_id.clone()),
                        Cell::from(image.size_formatted.clone()),
                        Cell::from(image.created_ago.clone()),
                        Cell::from(image.containers_count.clone()),
                    ])
                    .style(style)
                })
                .collect();

            // Create the table
            let table = Table::new(
                rows,
                vec![
                    Constraint::Percentage(40), // Repository:Tag
                    Constraint::Percentage(20), // Image ID
                    Constraint::Percentage(15), // Size
                    Constraint::Percentage(15), // Created
                    Constraint::Percentage(10), // Containers
                ],
            )
            .header(headers)
            .block(
                Block::default()
                    .title(format!("Images ({})", self.images.len()))
                    .borders(Borders::ALL),
            )
            .column_spacing(1);

            f.render_widget(table, area);
        }
    }

    fn render_inspect_modal(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        // Calculate modal size (80% of screen)
        let popup_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ])
            .split(area)[1];

        let popup_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ])
            .split(popup_area)[1];

        // Clear the background
        f.render_widget(Clear, popup_area);

        // Render modal content
        if let Some(inspect_data) = &self.inspect_data {
            let lines = self.format_inspect_data(inspect_data);

            // Create scrollable content
            let visible_lines: Vec<Line> = lines
                .into_iter()
                .skip(self.inspect_scroll)
                .take(popup_area.height.saturating_sub(3) as usize) // Leave space for border and help
                .collect();

            let content_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(1)])
                .split(popup_area);

            let paragraph = Paragraph::new(visible_lines)
                .block(
                    Block::default()
                        .title("Image Inspection")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .style(Style::default().fg(Color::White))
                .wrap(ratatui::widgets::Wrap { trim: false });

            f.render_widget(paragraph, content_area[0]);

            // Help text at bottom
            let help = Paragraph::new("[↑/↓] Scroll   [Esc] Close")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);

            f.render_widget(help, content_area[1]);
        } else {
            // Loading state
            let paragraph = Paragraph::new("Loading image details...")
                .block(
                    Block::default()
                        .title("Image Inspection")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);

            f.render_widget(paragraph, popup_area);
        }
    }

    fn format_inspect_data<'a>(&self, data: &'a ImageInspectDetails) -> Vec<Line<'a>> {
        let mut lines = Vec::new();

        // Basic Information
        lines.push(Line::from(vec![Span::styled(
            "Basic Information",
            Style::default().fg(Color::Yellow),
        )]));
        lines.push(Line::from(""));

        lines.push(Line::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::LightBlue)),
            Span::raw(&data.id),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Repository Tags: ", Style::default().fg(Color::LightBlue)),
            Span::raw(data.repo_tags.join(", ")),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Size: ", Style::default().fg(Color::LightBlue)),
            Span::raw(&data.size_formatted),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Created: ", Style::default().fg(Color::LightBlue)),
            Span::raw(&data.created_formatted),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Architecture: ", Style::default().fg(Color::LightBlue)),
            Span::raw(&data.architecture),
        ]));

        lines.push(Line::from(vec![
            Span::styled("OS: ", Style::default().fg(Color::LightBlue)),
            Span::raw(&data.os),
        ]));

        lines.push(Line::from(""));

        // Configuration
        lines.push(Line::from(vec![Span::styled(
            "Configuration",
            Style::default().fg(Color::Yellow),
        )]));
        lines.push(Line::from(""));

        if !data.env.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "Environment Variables:",
                Style::default().fg(Color::LightBlue),
            )]));
            for env in &data.env {
                lines.push(Line::from(vec![Span::raw("  "), Span::raw(env)]));
            }
            lines.push(Line::from(""));
        }

        if !data.exposed_ports.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Exposed Ports: ", Style::default().fg(Color::LightBlue)),
                Span::raw(data.exposed_ports.join(", ")),
            ]));
        }

        if !data.working_dir.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Working Directory: ", Style::default().fg(Color::LightBlue)),
                Span::raw(&data.working_dir),
            ]));
        }

        if !data.entrypoint.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Entrypoint: ", Style::default().fg(Color::LightBlue)),
                Span::raw(data.entrypoint.join(" ")),
            ]));
        }

        if !data.cmd.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Command: ", Style::default().fg(Color::LightBlue)),
                Span::raw(data.cmd.join(" ")),
            ]));
        }

        lines.push(Line::from(""));

        // Labels
        if !data.labels.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "Labels",
                Style::default().fg(Color::Yellow),
            )]));
            lines.push(Line::from(""));

            for (key, value) in &data.labels {
                lines.push(Line::from(vec![
                    Span::styled(format!("{}: ", key), Style::default().fg(Color::LightBlue)),
                    Span::raw(value),
                ]));
            }
        }

        lines
    }
}

#[async_trait]
impl Component for ImagesUI {
    fn name(&self) -> &str {
        "Images"
    }

    fn tab(&self) -> usize {
        self.tab_num
    }

    async fn start(&mut self) -> Result<()> {
        self.refresh_now().await
    }

    async fn tick(&mut self) {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_tick).as_secs() >= 10 {
            self.last_tick = now;
            let _ = self.refresh_now().await;
        }
    }

    async fn handle_input(&mut self, key: KeyCode) -> Result<()> {
        // Handle modal input first
        if self.show_inspect_modal {
            match key {
                KeyCode::Char('i') => {
                    self.show_inspect_modal = false;
                    self.inspect_data = None;
                    self.inspect_scroll = 0;
                }
                KeyCode::Up => {
                    if self.inspect_scroll > 0 {
                        self.inspect_scroll -= 1;
                    }
                }
                KeyCode::Down => {
                    if let Some(inspect_data) = &self.inspect_data {
                        let total_lines = self.format_inspect_data(inspect_data).len();
                        if self.inspect_scroll < total_lines.saturating_sub(10) {
                            self.inspect_scroll += 1;
                        }
                    }
                }
                _ => {}
            }
            return Ok(());
        }

        // Handle main table input
        match key {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_index < self.images.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
            }
            KeyCode::Char('r') | KeyCode::F(5) => {
                // Manual refresh for images only
                self.refresh_now().await?;
            }
            KeyCode::Char('d') => {
                if let Some(image) = self.get_selected_image() {
                    let image = image.clone();
                    self.delete_image(&image).await?;
                }
            }
            KeyCode::Char('p') => {
                if let Some(image) = self.get_selected_image() {
                    let image = image.clone();
                    self.pull_image(&image).await?;
                }
            }
            KeyCode::Char('i') => {
                if let Some(image) = self.get_selected_image() {
                    let image = image.clone();
                    self.inspect_image(&image).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        // Render main table
        self.render_main_table(f, area);

        // Render modal if active
        if self.show_inspect_modal {
            self.render_inspect_modal(f, area);
        }
    }

    fn render_help(&self) -> &'static str {
        if self.show_inspect_modal {
            "[↑/↓] Scroll   [Esc] Close"
        } else {
            "[↑/↓] Select   [D] Delete   [P] Pull   [I] Inspect   [R/F5] Refresh   [Q] Quit"
        }
    }
}
