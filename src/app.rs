use crate::{
    cli::Cli,
    reader::build_ftrace_tree_from_file,
    ui::{
        components::TraceTreeComponent,
        event::{Event, EventGenerator},
    },
};
use color_eyre::eyre::Result;
use crossterm::event::KeyCode;
use ratatui::{DefaultTerminal, crossterm};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct App {
    args: Cli,
    stopping: bool,
}

impl App {
    pub fn new(args: Cli) -> Self {
        Self {
            args,
            stopping: false,
        }
    }

    pub async fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let tree = build_ftrace_tree_from_file(&self.args.ftrace_path).await?;
        let mut tree_component = TraceTreeComponent::build_from_ftrace_tree(&tree);

        let mut event_generator = EventGenerator::new(Duration::from_millis(30));
        while !self.stopping {
            terminal
                .draw(|frame| {
                    tree_component.render(frame, frame.area());
                })
                .unwrap();
            let event = event_generator.poll_next().await;
            if let &Event::Key(key_event) = &event {
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.stopping = true;
                    }
                    _ => (),
                }
            }
            tree_component.handle_event(event);
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
            _ => {}
        }
    }
}
