use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use std::{
    fmt::{Debug, Formatter},
    io::{Stdout, stdout},
    panic::PanicHookInfo,
    sync::{Arc, Mutex, Weak},
    thread,
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
        let orig_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            panic_restore_terminal();
            orig_hook(info);
        }));
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
        let terminal = match self.terminal.get_mut() {
            Ok(terminal) => terminal,
            Err(poisoned) => poisoned.into_inner(),
        };
        disable_raw_mode().unwrap();
        crossterm::execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture,
        )
        .unwrap();
        terminal.show_cursor().unwrap();
        if !thread::panicking() {
            let _ = std::panic::take_hook();
        }
    }
}

fn panic_restore_terminal() {
    disable_raw_mode().ok();
    crossterm::execute!(stdout(), LeaveAlternateScreen).ok();
}
