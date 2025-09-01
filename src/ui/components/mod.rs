use crate::ui::event::Event;
use ratatui::{Frame, layout::Rect};

mod trace_tree;

pub use trace_tree::TraceTreeComponent;

pub trait Component<Msg> {
    fn handle(&mut self, event: Event<Msg>);

    fn render(&mut self, frame: &mut Frame, area: Rect);
}
