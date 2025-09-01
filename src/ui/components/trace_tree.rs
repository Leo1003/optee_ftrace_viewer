use super::Component;
use crate::{
    app::AppMsg,
    ftrace::{FtraceNode, FtraceTree},
    ui::event::Event,
};
use crossterm::event::{KeyCode, MouseButton, MouseEventKind};
use ratatui::{
    layout::{Alignment, Position, Rect}, style::{Color, Modifier, Style}, text::{Line, Span, Text}, widgets::{Block, BorderType, Borders}, Frame
};
use std::time::Duration;
use tui_tree_widget::{Tree, TreeItem, TreeState};

#[derive(Debug, Default)]
pub struct TraceTreeComponent {
    data: Vec<TreeItem<'static, u64>>,
    title: String,
    state: TreeState<u64>,
}

impl TraceTreeComponent {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            title: String::new(),
            state: TreeState::default(),
        }
    }

    pub fn with_ftrace_tree(tree: &FtraceTree) -> Self {
        TraceTreeComponent {
            data: Self::build_tree_data(tree),
            title: String::new(),
            state: TreeState::default(),
        }
    }

    pub fn build_tree_data(tree: &FtraceTree) -> Vec<TreeItem<'static, u64>> {
        let mut data = Vec::new();

        let time = tree
            .children()
            .map(|node| node.time().unwrap_or_default())
            .sum();
        for (child_id, children) in tree.children().enumerate() {
            data.push(build_ftrace_ui_tree(child_id as u64, children, time));
        }

        data
    }
}

impl Component<AppMsg> for TraceTreeComponent {
    fn handle(&mut self, event: Event<AppMsg>) {
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Up => {
                    self.state.key_up();
                }
                KeyCode::Down => {
                    self.state.key_down();
                }
                KeyCode::Left => {
                    self.state.key_left();
                }
                KeyCode::Right => {
                    self.state.key_right();
                }
                KeyCode::Home => {
                    self.state.select_first();
                }
                KeyCode::End => {
                    self.state.select_last();
                }
                KeyCode::Enter => {
                    self.state.toggle_selected();
                }
                KeyCode::PageUp => {
                    self.state.scroll_up(10);
                }
                KeyCode::PageDown => {
                    self.state.scroll_down(10);
                }
                _ => (),
            },
            Event::Mouse(mouse_event) => match mouse_event.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    self.state
                        .click_at(Position::new(mouse_event.column, mouse_event.row));
                }
                MouseEventKind::ScrollDown => {
                    self.state.scroll_down(3);
                }
                MouseEventKind::ScrollUp => {
                    self.state.scroll_up(3);
                }
                _ => (),
            },
            Event::Message(AppMsg::SetFtraceTitle(title)) => {
                self.title = format!(" {} ", title);
            }
            Event::Message(AppMsg::UpdateTree(tree_data)) => {
                self.data = tree_data;
                self.state = TreeState::default();
            }
            _ => (),
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .title(self.title.as_str())
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);
        let widget = Tree::new(&self.data)
            .unwrap()
            .block(block)
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED));
        frame.render_stateful_widget(widget, area, &mut self.state);
    }
}

fn build_ftrace_ui_tree(
    identifier: u64,
    node: &FtraceNode,
    upper_time: Duration,
) -> TreeItem<'static, u64> {
    let mut children_tree_items = Vec::new();
    let mut children_duration = Duration::ZERO;
    let time = node.time().unwrap_or_default();
    for (child_id, children) in node.children().enumerate() {
        children_tree_items.push(build_ftrace_ui_tree(child_id as u64, children, upper_time));
        children_duration += children.time().unwrap_or_default();
    }
    let self_time = time.saturating_sub(children_duration);

    let text = TraceLine {
        addr: node.func(),
        symbol: None,
        time,
        self_time,
        upper_time,
    };
    TreeItem::new(identifier, text, children_tree_items).unwrap()
}

const NAME_SPAN_STYLE: Style = Style::new().fg(Color::White);
const TIME_SPAN_STYLE: Style = Style::new().fg(Color::Yellow);
const SELF_TIME_SPAN_STYLE: Style = Style::new().fg(Color::DarkGray);
const RATIO_SPAN_STYLE: Style = Style::new().fg(Color::Blue);

#[derive(Clone, Debug)]
pub struct TraceLine {
    pub addr: u64,
    pub symbol: Option<String>,
    pub time: Duration,
    pub self_time: Duration,
    pub upper_time: Duration,
}

impl From<TraceLine> for Text<'static> {
    fn from(line: TraceLine) -> Self {
        let name_span = if let Some(symbol) = line.symbol {
            Span::styled(format!("{}()", symbol), NAME_SPAN_STYLE)
        } else {
            Span::styled(format!("0x{:016x}()", line.addr), NAME_SPAN_STYLE)
        };
        let time_span = Span::styled(format_duration(line.time), TIME_SPAN_STYLE);
        let self_time_span = Span::styled(
            format!("(self: {})", format_duration(line.self_time)),
            SELF_TIME_SPAN_STYLE,
        );
        let time_ratio = line.time.as_nanos() as f64 / line.upper_time.as_nanos() as f64 * 100.0;
        let ratio_span = Span::styled(format!("[{:.2}%]", time_ratio), RATIO_SPAN_STYLE);

        Line::from_iter([
            name_span,
            Span::raw(" "),
            time_span,
            Span::raw(" "),
            self_time_span,
            Span::raw(" "),
            ratio_span,
        ])
        .into()
    }
}

fn format_duration(duration: Duration) -> String {
    if duration.as_secs() > 0 {
        format!(
            "{:3}.{:03} s",
            duration.as_secs(),
            duration.subsec_millis() / 1000
        )
    } else if duration.as_millis() > 0 {
        format!(
            "{:3}.{:03} ms",
            duration.as_millis(),
            duration.subsec_micros() % 1000
        )
    } else if duration.as_micros() > 0 {
        format!(
            "{:3}.{:03} µs",
            duration.as_micros(),
            duration.subsec_nanos() % 1000
        )
    } else {
        format!("{} ns", duration.as_nanos())
    }
}
