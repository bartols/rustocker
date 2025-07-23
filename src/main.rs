mod app;
mod docker;
mod ui;
mod ui_containers;
mod ui_images;
mod ui_networks;
mod ui_volumes;

use app::App;
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize error handling
    color_eyre::install()?;

    // Create and run the app
    let mut app = App::new().await?;
    app.run().await?;

    Ok(())
}
