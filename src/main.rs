use clap::Parser as _;
use color_eyre::eyre::Result;

use crate::app::App;

mod app;
mod cli;
mod ftrace;
mod reader;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args = cli::Cli::parse();
    let terminal = ratatui::init();

    let mut app = App::new(args);
    let result = app.run(terminal).await;

    ratatui::restore();
    result
}
