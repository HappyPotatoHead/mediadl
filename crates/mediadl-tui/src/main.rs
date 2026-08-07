use crate::app::App;
use color_eyre::eyre::{Result, eyre};
use mediadl_core::config::load_or_create;

pub mod app;
pub mod event;
mod states;
pub mod traits;
pub mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let config = load_or_create().map_err(|e| eyre!(e))?;
    let terminal = ratatui::init();
    let result = App::new(config).run(terminal).await;
    ratatui::restore();
    result
}
