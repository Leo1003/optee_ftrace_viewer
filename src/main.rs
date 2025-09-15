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

#[allow(clippy::await_holding_lock)]
#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Cli::parse();
    let terminal_ctx = TerminalContext::get()?;
    let mut terminal_lock = terminal_ctx.terminal().lock().unwrap();

    let mut app = App::new(args);
    app.run(terminal_lock.deref_mut()).await
}
