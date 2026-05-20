mod app;
mod models;
pub mod adb;

use anyhow::Result;

pub use app::App;

pub async fn run() -> Result<()> {
    use ratatui::prelude::*;
    use ratatui::backend::CrosstermBackend;
    use std::io::stdout;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new();

    app.run(&mut terminal).await?;

    Ok(())
}