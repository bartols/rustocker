use crate::components::Component;
use crate::docker::DockerClient;
use crate::theme::current_theme;

use async_trait::async_trait;
use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct VolumesUI {
    tab_num: usize,
    docker_client: Arc<Mutex<DockerClient>>,
    selected_index: usize,
    volumes: Vec<String>,
    last_tick: std::time::Instant,
}

impl VolumesUI {
    pub fn new(docker_client: Arc<Mutex<DockerClient>>, tab_num: usize) -> Self {
        Self {
            tab_num,
            docker_client,
            selected_index: 0,
            volumes: Vec::new(),
            last_tick: std::time::Instant::now(),
        }
    }

    async fn refresh_now(&mut self) -> Result<()> {
        let client = self.docker_client.lock().await;
        match client.list_volumes().await {
            Ok(volumes) => {
                self.volumes = volumes;
                // Adjust selected index if necessary
                if self.selected_index >= self.volumes.len() && !self.volumes.is_empty() {
                    self.selected_index = self.volumes.len() - 1;
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to refresh volumes: {}", e);
                Err(e.into())
            }
        }
    }

    fn get_selected_volume(&self) -> Option<&String> {
        self.volumes.get(self.selected_index)
    }

    async fn delete_volume(&self, volume_name: &str) -> Result<()> {
        eprintln!("Deleting volume: {}", volume_name);
        // TODO: Implement volume deletion
        // Should check if volume is in use and ask for confirmation
        Ok(())
    }

    async fn create_volume(&self) -> Result<()> {
        eprintln!("Creating new volume...");
        // TODO: Implement volume creation
        // Should probably show a dialog to input volume name and options
        Ok(())
    }

    async fn inspect_volume(&self, volume_name: &str) -> Result<()> {
        eprintln!("Inspecting volume: {}", volume_name);
        // TODO: Implement volume inspection
        // Show detailed info including mountpoint and usage
        Ok(())
    }
}

#[async_trait]
impl Component for VolumesUI {
    fn name(&self) -> &str {
        "Volumes"
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

    async fn handle_input(&mut self, key: KeyCode) -> Result<bool> {
        match key {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                Ok(true)
            }
            KeyCode::Down => {
                if self.selected_index < self.volumes.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
                Ok(true)
            }
            KeyCode::Char('r') | KeyCode::F(5) => {
                // Manual refresh for volumes only
                self.refresh_now().await?;
                Ok(true)
            }
            KeyCode::Char('d') => {
                if let Some(volume_name) = self.get_selected_volume() {
                    self.delete_volume(volume_name).await?;
                }
                Ok(true)
            }
            KeyCode::Char('c') => {
                self.create_volume().await?;
                Ok(true)
            }
            KeyCode::Char('i') => {
                if let Some(volume_name) = self.get_selected_volume() {
                    self.inspect_volume(volume_name).await?;
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let theme = current_theme();

        if self.volumes.is_empty() {
            let paragraph = Paragraph::new("No volumes found or loading...")
                .block(
                    Block::default()
                        .title("Volumes")
                        .borders(Borders::ALL)
                        .border_style(theme.border_style()),
                )
                .style(theme.muted_style());
            f.render_widget(paragraph, area);
        } else {
            let items: Vec<ListItem> = self
                .volumes
                .iter()
                .enumerate()
                .map(|(i, volume)| {
                    let style = if i == self.selected_index {
                        theme.selected_style()
                    } else {
                        theme.normal_style()
                    };
                    ListItem::new(volume.clone()).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(format!("Volumes ({})", self.volumes.len()))
                        .borders(Borders::ALL)
                        .border_style(theme.border_style()),
                )
                .style(theme.normal_style());

            f.render_widget(list, area);
        }
    }

    fn render_help(&self) -> &'static str {
        "[↑/↓] Select   [C] Create   [D] Delete   [I] Inspect   [R/F5] Refresh   [Q] Quit"
    }
}
