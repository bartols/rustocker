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

use async_trait::async_trait;

pub struct ContainersUI {
    tab_num: usize,
    docker_client: Arc<Mutex<DockerClient>>,
    selected_index: usize,
    containers: Vec<String>,
    last_tick: std::time::Instant,
}

impl ContainersUI {
    pub fn new(docker_client: Arc<Mutex<DockerClient>>, tab_num: usize) -> Self {
        Self {
            tab_num,
            docker_client,
            selected_index: 0,
            containers: Vec::new(),
            last_tick: std::time::Instant::now(),
        }
    }

    pub async fn refresh_now(&mut self) -> Result<()> {
        let client = self.docker_client.lock().await;
        match client.list_containers().await {
            Ok(containers) => {
                self.containers = containers;
                // Adjust selected index if necessary
                if self.selected_index >= self.containers.len() && !self.containers.is_empty() {
                    self.selected_index = self.containers.len() - 1;
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to refresh containers: {}", e);
                Err(e.into())
            }
        }
    }

    fn get_selected_container(&self) -> Option<&String> {
        self.containers.get(self.selected_index)
    }

    async fn toggle_container_state(&self, container_name: &str) -> Result<()> {
        let client = self.docker_client.lock().await;

        // Get current status and toggle
        match client.get_container_status(container_name).await {
            Ok(status) => {
                if status.contains("Up") {
                    // Container is running, stop it
                    eprintln!("Stopping container: {}", container_name);
                    // TODO: Implement stop_container in DockerClient
                } else {
                    // Container is stopped, start it
                    eprintln!("Starting container: {}", container_name);
                    // TODO: Implement start_container in DockerClient
                }
            }
            Err(e) => {
                eprintln!("Failed to get container status: {}", e);
            }
        }

        Ok(())
    }

    async fn show_container_logs(&self, container_name: &str) -> Result<()> {
        eprintln!("Showing logs for container: {}", container_name);
        // TODO: Implement logs functionality
        // This could open a popup or new view with logs
        Ok(())
    }

    async fn delete_container(&self, container_name: &str) -> Result<()> {
        eprintln!("Deleting container: {}", container_name);
        // TODO: Implement container deletion
        // Should probably ask for confirmation first
        Ok(())
    }
}

#[async_trait]
impl Component for ContainersUI {
    fn name(&self) -> &str {
        "Containers"
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
                if self.selected_index < self.containers.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
                Ok(true)
            }
            KeyCode::Char('r') | KeyCode::F(5) => {
                self.refresh_now().await?;
                Ok(true)
            }
            KeyCode::Char('s') => {
                if let Some(container_name) = self.get_selected_container() {
                    self.toggle_container_state(container_name).await?;
                }
                Ok(true)
            }
            KeyCode::Char('l') => {
                if let Some(container_name) = self.get_selected_container() {
                    self.show_container_logs(container_name).await?;
                }
                Ok(true)
            }
            KeyCode::Char('d') => {
                if let Some(container_name) = self.get_selected_container() {
                    self.delete_container(container_name).await?;
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        if self.containers.is_empty() {
            // Show loading or empty state
            let paragraph = Paragraph::new("No containers found or loading...")
                .block(Block::default().title("Containers").borders(Borders::ALL))
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(paragraph, area);
        } else {
            // Create list items with selection highlighting
            let items: Vec<ListItem> = self
                .containers
                .iter()
                .enumerate()
                .map(|(i, container)| {
                    let style = if i == self.selected_index {
                        Style::default().fg(Color::LightYellow).bg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    ListItem::new(container.clone()).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(format!("Containers ({})", self.containers.len()))
                        .borders(Borders::ALL),
                )
                .style(Style::default());

            f.render_widget(list, area);
        }
    }

    fn render_help(&self) -> &'static str {
        "[↑/↓] Select   [S] Start/Stop   [L] Logs   [D] Delete   [R/F5] Refresh   [Q] Quit"
    }
}
