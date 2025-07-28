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

pub struct NetworksUI {
    tab_num: usize,
    docker_client: Arc<Mutex<DockerClient>>,
    selected_index: usize,
    networks: Vec<String>,
    cancellation_token: CancellationToken,
}

impl NetworksUI {
    pub fn new(docker_client: Arc<Mutex<DockerClient>>, tab_num: usize) -> Self {
        Self {
            tab_num,
            docker_client,
            selected_index: 0,
            networks: Vec::new(),
            cancellation_token: CancellationToken::new(),
        }
    }

    async fn refresh_now(&mut self) -> Result<()> {
        let client = self.docker_client.lock().await;
        match client.list_networks().await {
            Ok(networks) => {
                self.networks = networks;
                // Adjust selected index if necessary
                if self.selected_index >= self.networks.len() && !self.networks.is_empty() {
                    self.selected_index = self.networks.len() - 1;
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to refresh networks: {}", e);
                Err(e.into())
            }
        }
    }

    fn get_selected_network(&self) -> Option<&String> {
        self.networks.get(self.selected_index)
    }

    async fn delete_network(&self, network_name: &str) -> Result<()> {
        eprintln!("Deleting network: {}", network_name);
        // TODO: Implement network deletion
        // Should check if network is in use and ask for confirmation
        Ok(())
    }

    async fn create_network(&self) -> Result<()> {
        eprintln!("Creating new network...");
        // TODO: Implement network creation
        // Should probably show a dialog to input network name and options
        Ok(())
    }

    async fn inspect_network(&self, network_name: &str) -> Result<()> {
        eprintln!("Inspecting network: {}", network_name);
        // TODO: Implement network inspection
        // Show detailed info including connected containers
        Ok(())
    }
}

impl Component for NetworksUI {
    fn name(&self) -> &str {
        "Networks"
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
            // Set up refresh interval (networks refresh every 20 seconds - infrequent)
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(20));

            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        break;
                    }
                    _ = interval.tick() => {
                        if let Err(e) = docker_client.lock().await.list_networks().await {
                            eprintln!("Failed to refresh networks: {}", e);
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
                if self.selected_index < self.networks.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
            }
            KeyCode::Char('r') | KeyCode::F(5) => {
                // Manual refresh for networks only
                self.refresh_now().await?;
            }
            KeyCode::Char('d') => {
                if let Some(network_name) = self.get_selected_network() {
                    self.delete_network(network_name).await?;
                }
            }
            KeyCode::Char('c') => {
                self.create_network().await?;
            }
            KeyCode::Char('i') => {
                if let Some(network_name) = self.get_selected_network() {
                    self.inspect_network(network_name).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        if self.networks.is_empty() {
            let paragraph = Paragraph::new("No networks found or loading...")
                .block(Block::default().title("Networks").borders(Borders::ALL))
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(paragraph, area);
        } else {
            let items: Vec<ListItem> = self
                .networks
                .iter()
                .enumerate()
                .map(|(i, network)| {
                    let style = if i == self.selected_index {
                        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    ListItem::new(network.clone()).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(format!("Networks ({})", self.networks.len()))
                        .borders(Borders::ALL),
                )
                .style(Style::default());

            f.render_widget(list, area);
        }
    }

    fn render_help() -> &'static str {
        "[↑/↓] Select   [C] Create   [D] Delete   [I] Inspect   [R/F5] Refresh   [Q] Quit"
    }
}
