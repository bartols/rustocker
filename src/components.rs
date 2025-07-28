use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::Frame;

pub(crate) trait Component {
    fn name(&self) -> &str;

    fn tab(&self) -> usize;

    async fn start(&mut self) -> Result<()>;
    async fn handle_input(&mut self, key: KeyCode) -> Result<()>;

    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect);
    fn render_help() -> &'static str;
}
