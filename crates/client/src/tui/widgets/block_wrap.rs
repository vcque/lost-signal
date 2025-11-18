use ratatui::{
    layout::Rect,
    widgets::{Block, Widget},
};

pub trait BlockWrap<'a> {
    fn wrap<T: Widget>(self, widget: T) -> WBlock<'a, T>;
}

pub struct WBlock<'a, T> {
    widget: T,
    block: Block<'a>,
}

impl<'a, T: Widget> Widget for WBlock<'a, T> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let Self { widget, block } = self;

        let inner = block.inner(area);
        widget.render(inner, buf);
        block.render(area, buf);
    }
}

impl<'a> BlockWrap<'a> for Block<'a> {
    fn wrap<T: Widget>(self, widget: T) -> WBlock<'a, T> {
        WBlock {
            widget,
            block: self,
        }
    }
}

