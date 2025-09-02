use crate::{
    cli::Cli,
    ftrace::FtraceNode,
    reader::build_ftrace_tree_from_file,
    symbol::{info::SymbolInfo, resolver::SymbolResolver},
    ui::{
        components::{Component as _, TraceTreeComponent},
        event::{Event, EventGenerator},
    },
};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{DefaultTerminal, crossterm};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::sync::mpsc::UnboundedSender;
use tui_tree_widget::TreeItem;

#[derive(Debug)]
pub struct App {
    args: Cli,
    stopping: bool,
    event_generator: EventGenerator<AppMsg>,
}

impl App {
    pub fn new(args: Cli) -> Self {
        Self {
            args,
            stopping: false,
            event_generator: EventGenerator::new(Duration::from_millis(30)),
        }
    }

    pub async fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let ftrace_file = self.args.ftrace_path.clone();
        let sources = self.args.elf.clone();
        let event_sender = self.event_generator.get_app_event_sender();
        tokio::spawn(async move { initialize_ftrace(&ftrace_file, sources, event_sender).await });
        let mut tree_component = TraceTreeComponent::new();

        while !self.stopping {
            terminal
                .draw(|frame| {
                    tree_component.render(frame, frame.area());
                })
                .unwrap();
            let event = self.event_generator.poll_next().await;
            if let &Event::Key(key_event) = &event {
                self.handle_key_event(key_event);
            }
            tree_component.handle(event);
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.stopping = true;
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug)]
pub enum AppMsg {
    SetFtraceTitle(String),
    UpdateTree(Vec<TreeItem<'static, u64>>),
}

async fn initialize_ftrace(
    ftrace_file: &Path,
    sources: Vec<PathBuf>,
    event_sender: UnboundedSender<AppMsg>,
) -> Result<()> {
    let mut tree = build_ftrace_tree_from_file(ftrace_file).await?;
    let symbol_info: SymbolInfo = tree.trace_info().parse()?;
    let mut resolver = SymbolResolver::new(sources);
    for node in tree.children_mut() {
        recursive_resolve_symbol(&mut resolver, &symbol_info, node);
    }
    let tree_data = TraceTreeComponent::build_tree_data(&tree);
    event_sender.send(AppMsg::SetFtraceTitle(symbol_info.title.clone()))?;
    event_sender.send(AppMsg::UpdateTree(tree_data))?;
    Ok(())
}

fn recursive_resolve_symbol(
    resolver: &mut SymbolResolver,
    symbol_info: &SymbolInfo,
    node: &mut FtraceNode,
) {
    if let Some(symbol) = resolve_symbol(resolver, symbol_info, node.func()) {
        node.set_symbol(symbol);
    }

    for child in node.children_mut() {
        recursive_resolve_symbol(resolver, symbol_info, child);
    }
}

fn resolve_symbol(
    resolver: &mut SymbolResolver,
    symbol_info: &SymbolInfo,
    addr: u64,
) -> Option<String> {
    let load_info = symbol_info.find_by_addr(addr)?;
    let reladdr = load_info.calculate_reladdr(addr)?;
    resolver.resolve_symbol(&load_info, reladdr)
}
