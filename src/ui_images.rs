use crate::components::Component;
use crate::docker::DockerClient;
use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub struct ImagesUI {
    tab_num: usize,
    docker_client: Arc<Mutex<DockerClient>>,
    selected_index: usize,
    images: Vec<String>,
    cancellation_token: CancellationToken,
}

impl ImagesUI {
    pub fn new(docker_client: Arc<Mutex<DockerClient>>, tab_num: usize) -> Self {
        Self {
            tab_num,
            docker_client,
            selected_index: 0,
            images: Vec::new(),
            cancellation_token: CancellationToken::new(),
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

    fn get_selected_image(&self) -> Option<&String> {
        self.images.get(self.selected_index)
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

impl Component for ImagesUI {
    fn name(&self) -> &str {
        "Images"
    }

    fn tab(&self) -> usize {
        self.tab_num
    }

    async fn start(&mut self) -> Result<()> {
        let docker_client = Arc::clone(&self.docker_client);
        let cancellation_token = self.cancellation_token.clone();

        // Initial load
        self.refresh_now().await?;

        tokio::spawn(async move {
            // Set up refresh interval (images refresh every 15 seconds - less frequent)
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(15));
            interval.reset();

            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        break;
                    }
                    _ = interval.tick() => {
                        if let Err(e) = docker_client.lock().await.list_images().await {
                            eprintln!("Failed to refresh images: {}", e);
                        }
                        // Note: Background refresh only logs errors
                        // Manual refresh updates the UI data
                    }
                }
            }
        });

        Ok(())
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
                    self.delete_image(image_name).await?;
                }
            }
            KeyCode::Char('p') => {
                if let Some(image_name) = self.get_selected_image() {
                    self.pull_image(image_name).await?;
                }
            }
            KeyCode::Char('i') => {
                if let Some(image_name) = self.get_selected_image() {
                    self.inspect_image(image_name).await?;
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
                        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    ListItem::new(image.clone()).style(style)
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

    fn render_help() -> &'static str {
        "[↑/↓] Select   [D] Delete   [P] Pull   [I] Inspect   [R/F5] Refresh   [Q] Quit"
    }
}
