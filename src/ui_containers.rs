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

pub struct ContainersUI {
    tab_num: usize,
    docker_client: Arc<Mutex<DockerClient>>,
    selected_index: usize,
    containers: Vec<String>,
    cancellation_token: CancellationToken,
}

impl ContainersUI {
    pub fn new(docker_client: Arc<Mutex<DockerClient>>, tab_num: usize) -> Self {
        Self {
            tab_num,
            docker_client,
            selected_index: 0,
            containers: Vec::new(),
            cancellation_token: CancellationToken::new(),
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

impl Component for ContainersUI {
    fn name(&self) -> &str {
        "Containers"
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
            // Set up refresh interval (containers refresh every 5 seconds)
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        break;
                    }
                    _ = interval.tick() => {
                        if let Err(e) = docker_client.lock().await.list_containers().await {
                            eprintln!("Failed to refresh containers: {}", e);
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
                if self.selected_index < self.containers.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
            }
            KeyCode::Char('r') | KeyCode::F(5) => {
                self.refresh_now().await?;
            }
            KeyCode::Char('s') => {
                if let Some(container_name) = self.get_selected_container() {
                    self.toggle_container_state(container_name).await?;
                }
            }
            KeyCode::Char('l') => {
                if let Some(container_name) = self.get_selected_container() {
                    self.show_container_logs(container_name).await?;
                }
            }
            KeyCode::Char('d') => {
                if let Some(container_name) = self.get_selected_container() {
                    self.delete_container(container_name).await?;
                }
            }
            _ => {}
        }
        Ok(())
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
                        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
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

    fn render_help() -> &'static str {
        "[↑/↓] Select   [S] Start/Stop   [L] Logs   [D] Delete   [R/F5] Refresh   [Q] Quit"
    }
}
