use crate::docker::DockerClient;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use futures::{FutureExt, StreamExt};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub enum AppEvent {
    // Key events
    Key(KeyEvent),
    // Docker events
    ContainersUpdated(Vec<String>),
    ImagesUpdated(Vec<String>),
    NetworksUpdated(Vec<String>),
    VolumesUpdated(Vec<String>),
    // Manual refresh
    RefreshRequested,
    // Error events
    Error(String),
}

pub struct App {
    pub active_tab: usize,
    pub should_quit: bool,
    // Docker data
    pub containers: Vec<String>,
    pub images: Vec<String>,
    pub networks: Vec<String>,
    pub volumes: Vec<String>,
    // Event handling
    event_rx: mpsc::UnboundedReceiver<AppEvent>,
    event_tx: mpsc::UnboundedSender<AppEvent>,
    cancellation_token: CancellationToken,
}

impl App {
    pub async fn new() -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();

        Ok(Self {
            active_tab: 0,
            should_quit: false,
            containers: Vec::new(),
            images: Vec::new(),
            networks: Vec::new(),
            volumes: Vec::new(),
            event_rx,
            event_tx,
            cancellation_token,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Initialize terminal
        let mut terminal = self.init_terminal()?;

        // Start background tasks
        self.start_docker_task().await?;
        self.start_input_task()?;

        // Main event loop
        while !self.should_quit {
            // Draw the UI
            terminal.draw(|frame| crate::ui::draw_ui(frame, self))?;

            // Handle events
            if let Some(event) = self.event_rx.recv().await {
                self.handle_event(event).await?;
            }
        }

        // Cleanup
        self.cleanup_terminal(&mut terminal)?;
        self.cancellation_token.cancel();

        Ok(())
    }

    async fn handle_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::Key(key) => self.handle_key_event(key),
            AppEvent::ContainersUpdated(containers) => {
                self.containers = containers;
            }
            AppEvent::ImagesUpdated(images) => {
                self.images = images;
            }
            AppEvent::NetworksUpdated(networks) => {
                self.networks = networks;
            }
            AppEvent::VolumesUpdated(volumes) => {
                self.volumes = volumes;
            }
            AppEvent::RefreshRequested => {
                // Create a new docker client and fetch data immediately
                tokio::spawn({
                    let event_tx = self.event_tx.clone();
                    async move {
                        if let Ok(docker_client) = DockerClient::new().await {
                            Self::fetch_all_docker_data(&docker_client, &event_tx).await;
                        }
                    }
                });
            }
            AppEvent::Error(error) => {
                // Log error or show in UI
                eprintln!("Docker error: {}", error);
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('r') => {
                // Manual refresh with 'r' key
                let _ = self.event_tx.send(AppEvent::RefreshRequested);
            }
            KeyCode::F(5) => {
                // Manual refresh with F5 key
                let _ = self.event_tx.send(AppEvent::RefreshRequested);
            }
            KeyCode::Right => {
                self.active_tab = (self.active_tab + 1) % 4;
            }
            KeyCode::Left => {
                if self.active_tab == 0 {
                    self.active_tab = 3;
                } else {
                    self.active_tab -= 1;
                }
            }
            _ => {}
        }
    }

    async fn start_docker_task(&self) -> Result<()> {
        let docker_client = DockerClient::new().await?;
        let event_tx = self.event_tx.clone();
        let cancellation_token = self.cancellation_token.clone();

        tokio::spawn(async move {
            // Load initial data
            Self::fetch_all_docker_data(&docker_client, &event_tx).await;

            // Set up periodic refresh (every 10 seconds)
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        break;
                    }
                    _ = interval.tick() => {
                        Self::fetch_all_docker_data(&docker_client, &event_tx).await;
                    }
                }
            }
        });

        Ok(())
    }

    async fn fetch_all_docker_data(
        docker_client: &DockerClient,
        event_tx: &mpsc::UnboundedSender<AppEvent>,
    ) {
        // Fetch containers
        match docker_client.list_containers().await {
            Ok(containers) => {
                let _ = event_tx.send(AppEvent::ContainersUpdated(containers));
            }
            Err(e) => {
                let _ = event_tx.send(AppEvent::Error(format!("Failed to list containers: {}", e)));
            }
        }

        // Fetch images
        match docker_client.list_images().await {
            Ok(images) => {
                let _ = event_tx.send(AppEvent::ImagesUpdated(images));
            }
            Err(e) => {
                let _ = event_tx.send(AppEvent::Error(format!("Failed to list images: {}", e)));
            }
        }

        // Fetch networks
        match docker_client.list_networks().await {
            Ok(networks) => {
                let _ = event_tx.send(AppEvent::NetworksUpdated(networks));
            }
            Err(e) => {
                let _ = event_tx.send(AppEvent::Error(format!("Failed to list networks: {}", e)));
            }
        }

        // Fetch volumes
        match docker_client.list_volumes().await {
            Ok(volumes) => {
                let _ = event_tx.send(AppEvent::VolumesUpdated(volumes));
            }
            Err(e) => {
                let _ = event_tx.send(AppEvent::Error(format!("Failed to list volumes: {}", e)));
            }
        }
    }

    fn start_input_task(&self) -> Result<()> {
        let event_tx = self.event_tx.clone();
        let cancellation_token = self.cancellation_token.clone();

        tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();

            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        break;
                    }
                    maybe_event = reader.next().fuse() => {
                        match maybe_event {
                            Some(Ok(crossterm::event::Event::Key(key))) => {
                                let _ = event_tx.send(AppEvent::Key(key));
                            }
                            Some(Err(_)) => {
                                // Handle error if needed
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        Ok(())
    }

    fn init_terminal(&self) -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(io::stdout());
        Ok(Terminal::new(backend)?)
    }

    fn cleanup_terminal(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<()> {
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen
        )?;
        terminal.show_cursor()?;
        Ok(())
    }
}
