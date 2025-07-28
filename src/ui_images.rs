use crate::components::Component;
use crate::docker::DockerClient;

use bollard::{image, models::ImageSummary};

use async_trait::async_trait;
use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::sync::Arc;
use tokio::sync::Mutex;

fn get_name(image: &ImageSummary) -> String {
    let tags = &image.repo_tags;
    if !tags.is_empty() && tags[0] != "<none>:<none>" {
        tags[0].clone()
    } else {
        // Use first 12 chars of image ID as fallback
        let id = image.id.clone();
        if id.len() > 12 {
            format!("{}...", &id[7..19]) // Skip "sha256:" prefix
        } else {
            id
        }
    }
}

pub struct ImagesUI {
    tab_num: usize,
    docker_client: Arc<Mutex<DockerClient>>,
    selected_index: usize,
    images: Vec<ImageSummary>,
    last_tick: std::time::Instant,
}

impl ImagesUI {
    pub fn new(docker_client: Arc<Mutex<DockerClient>>, tab_num: usize) -> Self {
        Self {
            tab_num,
            docker_client,
            selected_index: 0,
            images: Vec::new(),
            last_tick: std::time::Instant::now(),
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

    fn get_selected_image(&self) -> Option<String> {
        self.images
            .get(self.selected_index)
            .map(|img| get_name(img))
    }

    async fn delete_image(&self, image_name: &str) -> Result<()> {
        eprintln!("Deleting image: {}", image_name);
        // TODO: Implement image deletion
        // Should ask for confirmation and handle dependencies
        Ok(())
    }

    async fn pull_image(&self, image_name: &str) -> Result<()> {
        eprintln!("Pulling image: {}", image_name);
        // TODO: Implement image pull
        // Should show progress if possible
        Ok(())
    }

    async fn inspect_image(&self, image_name: &str) -> Result<()> {
        eprintln!("Inspecting image: {}", image_name);
        // TODO: Implement image inspection
        // Show detailed info in a popup or new view
        Ok(())
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
                if let Some(image_name) = self.get_selected_image() {
                    self.delete_image(&image_name).await?;
                }
            }
            KeyCode::Char('p') => {
                if let Some(image_name) = self.get_selected_image() {
                    self.pull_image(&image_name).await?;
                }
            }
            KeyCode::Char('i') => {
                if let Some(image_name) = self.get_selected_image() {
                    self.inspect_image(&image_name).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        if self.images.is_empty() {
            let paragraph = Paragraph::new("No images found or loading...")
                .block(Block::default().title("Images").borders(Borders::ALL))
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(paragraph, area);
        } else {
            let items: Vec<ListItem> = self
                .images
                .iter()
                .enumerate()
                .map(|(i, image)| {
                    let style = if i == self.selected_index {
                        Style::default().fg(Color::LightYellow).bg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    ListItem::new(get_name(image)).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(format!("Images ({})", self.images.len()))
                        .borders(Borders::ALL),
                )
                .style(Style::default());

            f.render_widget(list, area);
        }
    }

    fn render_help(&self) -> &'static str {
        "[↑/↓] Select   [D] Delete   [P] Pull   [I] Inspect   [R/F5] Refresh   [Q] Quit"
    }
}
