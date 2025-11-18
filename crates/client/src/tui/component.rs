use ratatui::{buffer::Buffer, layout::Rect};
use crate::tui_adapter::Event;

pub trait Component {
    type State;
    fn on_event(self, event: &Event, state: &mut Self::State) -> bool;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State);
}