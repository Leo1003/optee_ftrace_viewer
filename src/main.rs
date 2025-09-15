use crate::{app::App, cli::Cli, ui::term::TerminalContext};
use clap::Parser as _;
use color_eyre::eyre::Result;
use std::ops::DerefMut;

mod app;
mod cli;
mod ftrace;
mod reader;
mod symbol;
mod ui;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Cli::parse();
    tui_main(args).await
}

#[allow(clippy::await_holding_lock)]
async fn tui_main(args: Cli) -> Result<()> {
    let terminal_ctx = TerminalContext::get()?;
    let mut terminal_lock = terminal_ctx.terminal().lock().unwrap();

    let mut app = App::new(args);
    app.run(terminal_lock.deref_mut()).await
}
