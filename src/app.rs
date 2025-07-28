use crate::components::Component;
use crate::docker::DockerClient;
use crate::{
    ui_containers::ContainersUI, ui_images::ImagesUI, ui_networks::NetworksUI,
    ui_volumes::VolumesUI,
};

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use futures::{FutureExt, StreamExt};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, sync::Arc};
use tokio::sync::{Mutex, mpsc};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub enum AppEvent {
    // Key events
    Key(KeyEvent),
    // Error events (only global errors now)
    Error(String),
}

pub struct App {
    pub active_tab: usize,
    pub should_quit: bool,
    // UI modules
    pub components: Vec<Box<dyn Component>>,
    // Event handling
    event_rx: mpsc::UnboundedReceiver<AppEvent>,
    event_tx: mpsc::UnboundedSender<AppEvent>,
    cancellation_token: CancellationToken,
}

impl App {
    pub async fn new() -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();

        // Initialize shared Docker client
        let docker_client = Arc::new(Mutex::new(DockerClient::new().await?));

        // Initialize UI modules with shared Docker client
        let containers_ui = ContainersUI::new(Arc::clone(&docker_client), 0);
        let images_ui = ImagesUI::new(Arc::clone(&docker_client), 1);
        let networks_ui = NetworksUI::new(Arc::clone(&docker_client), 2);
        let volumes_ui = VolumesUI::new(docker_client, 3);

        let components: Vec<Box<dyn Component>> = vec![
            Box::new(containers_ui),
            Box::new(images_ui),
            Box::new(networks_ui),
            Box::new(volumes_ui),
        ];

        Ok(Self {
            active_tab: 0,
            should_quit: false,
            components,
            event_rx,
            event_tx,
            cancellation_token,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Initialize terminal
        let mut terminal = self.init_terminal()?;

        // Start background refresh tasks for each UI module
        for component in &mut self.components {
            component.start().await?;
        }

        // Start input task
        self.start_input_task()?;

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

        // Main event loop
        while !self.should_quit {
            // Draw the UI
            terminal.draw(|frame| crate::ui::draw_ui(frame, self))?;

            tokio::select! {
                _ = interval.tick() => {
                    for component in &mut self.components {
                        component.tick().await;
                    }
                }

                // Handle events
                event = self.event_rx.recv() => {
                    if let Some(event) = event {
                        self.handle_event(event).await?;
                    }
                }
            }
        }

        // Cleanup
        self.cleanup_terminal(&mut terminal)?;
        self.cancellation_token.cancel();

        Ok(())
    }

    async fn handle_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::Key(key) => {
                // First try to delegate to active UI module
                let mut handled = false;
                if let Some(component) = self
                    .components
                    .iter_mut()
                    .find(|c| c.tab() == self.active_tab)
                {
                    // Component returns true if it handled the event
                    handled = component.handle_input(key.code).await.unwrap_or(false);
                }

                // If component didn't handle it, try global keys
                if !handled {
                    self.handle_global_key_event(key);
                }
            }
            AppEvent::Error(error) => {
                // Log global errors
                eprintln!("Application error: {}", error);
            }
        }
        Ok(())
    }

    fn handle_global_key_event(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                true
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                true
            }
            KeyCode::Esc => {
                self.should_quit = true;
                true
            }
            KeyCode::Right => {
                self.active_tab = (self.active_tab + 1) % self.components.len();
                true
            }
            KeyCode::Left => {
                if self.active_tab == 0 {
                    self.active_tab = self.components.len() - 1;
                } else {
                    self.active_tab -= 1;
                }
                true
            }
            _ => false, // Not handled globally
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
