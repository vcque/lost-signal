use losig_core::types::{StageTurn, Timeline, TimelineType};
use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Widget, block::Title},
};

use crate::{tui::THEME, world::WorldView};

pub struct TimelineWidget {
    timeline: Timeline,
    current: StageTurn,
    stage_name: String,
    timeline_type: TimelineType,
}

impl TimelineWidget {
    pub fn new(world: &WorldView) -> Self {
        Self {
            timeline: world.timeline,
            current: world.stage_turn,
            stage_name: world.stage_info.name.clone(),
            timeline_type: world.stage_info.timeline_type,
        }
    }

    fn as_line(&self) -> Line<'static> {
        let stage_span = Span::from(self.stage_name.to_string());

        // For Immediate timeline type, only show the stage name
        if self.timeline_type == TimelineType::Immediate {
            return Line::from(vec![stage_span]);
        }

        // For Asynchronous timeline type, show the full timeline with turns
        let turn_span = Span::from(format!(" - Turn {}: ", self.current));

        let mut timelines_spans: Vec<Span> = vec![stage_span, turn_span];
        let chars_before = self.current.saturating_sub(self.timeline.tail).div_ceil(5);

        for i in (0..chars_before).rev() {
            timelines_spans.push(Span::from(" ").bg(tail_color(i)));
        }

        timelines_spans.push(Span::from("@").fg(Color::Black).bg(Color::White));
        let chars_after = self.timeline.head.saturating_sub(self.current).div_ceil(5);
        for i in 0..chars_after {
            timelines_spans.push(Span::from(" ").bg(head_color(i)));
        }

        Line::from(timelines_spans)
    }
}

impl Widget for TimelineWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.as_line().render(area, buf);
    }
}

fn head_color(i: u64) -> Color {
    let mut color = THEME.palette.timeline_head;
    color.lightness = lightness(i);
    Color::from_hsl(color)
}

fn tail_color(i: u64) -> Color {
    let mut color = THEME.palette.timeline_tail;
    color.lightness = lightness(i);
    Color::from_hsl(color)
}

fn lightness(i: u64) -> f32 {
    1.0 - 0.05 * (i + 1) as f32
}

impl<'a> From<TimelineWidget> for Title<'a> {
    fn from(val: TimelineWidget) -> Self {
        Title {
            content: val.as_line(),
            alignment: None,
            position: None,
        }
    }
}
