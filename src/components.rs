use async_trait::async_trait;
use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::Frame;

#[async_trait]
pub(crate) trait Component {
    fn name(&self) -> &str;

    fn tab(&self) -> usize;

    async fn start(&mut self) -> Result<()>;
    async fn tick(&mut self);
    async fn handle_input(&mut self, key: KeyCode) -> Result<bool>;

    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect);
    fn render_help(&self) -> &'static str;
}
