mod app;
mod components;
mod docker;
mod theme;
mod ui;
mod ui_containers;
mod ui_images;
mod ui_networks;
mod ui_volumes;

use app::App;
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<String>>();

    // Initialize error handling
    color_eyre::install()?;

    theme::init_theme(theme::Theme::blue());

    let ip = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };

    // Create and run the app
    let mut app = App::new(ip).await?;
    app.run().await?;

    Ok(())
}
