use crate::{cli::Cli, reader::build_ftrace_tree_from_file};
use color_eyre::eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt as _;
use ratatui::{crossterm, layout::Alignment, widgets::{Block, BorderType, Paragraph}, DefaultTerminal};
use std::{fmt::Write as _, time::Duration};

#[derive(Clone, Debug)]
pub struct App {
    args: Cli,
    stopping: bool,
    text: String,
    scroll: u16,
}

impl App {
    pub fn new(args: Cli) -> Self {
        Self {
            args,
            stopping: false,
            text: String::new(),
            scroll: 0,
        }
    }

    pub async fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let tree = build_ftrace_tree_from_file(&self.args.ftrace_path).await?;
        let mut buf = String::new();
        for node in tree.dfs_iter() {
            writeln!(
                buf,
                "{}0x{:016x}(): {:?}",
                "  ".repeat(node.depth() as usize),
                node.func(),
                node.time(),
            )
            .ok();
        }
        self.text = buf;

        let mut event_stream = EventStream::new();
        while !self.stopping {
            terminal.draw(|frame| {
                let block = Block::bordered()
                    .title("optee-ftrace-viewer")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded);
                let widget = Paragraph::new(self.text.as_str())
                    .block(block)
                    .alignment(Alignment::Left)
                    .scroll((self.scroll, 0));
                frame.render_widget(widget, frame.area());
            })?;
            tokio::select! {
                Some(Ok(evt)) = event_stream.next() => {
                    #[allow(clippy::single_match)]
                    match evt {
                        Event::Key(key_event) => {
                            self.handle_key_event(key_event);
                        }
                        _ => {}
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    // Just to refresh the UI periodically
                }
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => {
                self.stopping = true;
            }
            KeyCode::Esc => {
                self.stopping = true;
            }
            KeyCode::Up => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::Down => {
                self.scroll = self.scroll.saturating_add(1);
            }
            _ => {}
        }
    }
}
