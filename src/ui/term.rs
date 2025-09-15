use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use std::{
    io::{Stdout, stdout},
    sync::{Arc, Mutex, Weak},
};

static GLOBAL_TERMINAL_CONTEXT: Mutex<Weak<TerminalContext>> = Mutex::new(Weak::new());

#[derive(Debug)]
pub struct TerminalContext {
    terminal: Mutex<Terminal<CrosstermBackend<Stdout>>>,
}

impl TerminalContext {
    pub fn get() -> std::io::Result<Arc<Self>> {
        let mut guard = GLOBAL_TERMINAL_CONTEXT.lock().unwrap();
        if let Some(ctx) = guard.upgrade() {
            Ok(ctx)
        } else {
            let ctx = Arc::new(Self::new()?);
            *guard = Arc::downgrade(&ctx);
            Ok(ctx)
        }
    }

    fn new() -> std::io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        Ok(Self {
            terminal: Mutex::new(Terminal::new(CrosstermBackend::new(stdout))?),
        })
    }

    pub fn terminal(&self) -> &Mutex<Terminal<CrosstermBackend<Stdout>>> {
        &self.terminal
    }
}

impl Drop for TerminalContext {
    fn drop(&mut self) {
        let terminal = self.terminal.get_mut().unwrap();
        disable_raw_mode().unwrap();
        crossterm::execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture,
        )
        .unwrap();
        terminal.show_cursor().unwrap();
    }
}
