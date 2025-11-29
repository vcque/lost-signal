use bounded_integer::BoundedU8;
use losig_core::sense::{SenseStrength, Senses, SensesInfo};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::tui::THEME;

struct SenseWidget<'a> {
    label: &'a str,
    indicator: &'a str,
    status: Option<Line<'a>>,
    selected: bool,
    active: bool,
}

impl<'a> Widget for SenseWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let Self {
            label,
            indicator,
            status,
            active,
            selected,
        } = self;

        let layout = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]);
        let [first, second] = layout.areas(area);

        let first_line_style = match (selected, active) {
            (true, _) => THEME.palette.ui_selected,
            (_, true) => THEME.palette.ui_highlight,
            _ => THEME.palette.ui_disabled,
        };

        buf.set_string(
            first.x,
            first.y,
            ".".repeat(area.width as usize),
            first_line_style,
        );
        Line::from(label).style(first_line_style).render(first, buf);
        Line::from(indicator)
            .style(first_line_style)
            .right_aligned()
            .render(first, buf);

        let status = status.unwrap_or(Line::from("-").style(THEME.palette.ui_disabled));
        status.right_aligned().render(second, buf);
    }
}

pub struct SelfSenseWidget<'a> {
    pub sense: bool,
    pub info: Option<&'a losig_core::sense::SelfInfo>,
    pub selected: bool,
}

impl<'a> Widget for SelfSenseWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]);
        let [first, second] = layout.areas(area);

        // Render first line (label and indicator)
        let first_line_style = match (self.selected, self.sense) {
            (true, _) => THEME.palette.ui_selected,
            (_, true) => THEME.palette.ui_highlight,
            _ => THEME.palette.ui_disabled,
        };

        buf.set_string(
            first.x,
            first.y,
            ".".repeat(area.width as usize),
            first_line_style,
        );
        Line::from("Self")
            .style(first_line_style)
            .render(first, buf);
        let indicator = if self.sense { "(+)" } else { "(-)" };
        Line::from(indicator)
            .style(first_line_style)
            .right_aligned()
            .render(first, buf);

        // Render second line (HP gauge + Focus)
        if let Some(info) = self.info {
            let hp = info.hp.min(10);
            let hp_max = info.hp_max.min(10);

            // Render "HP: " label
            buf.set_string(second.x, second.y, "HP: ", Style::default());

            // Render HP gauge manually (reversed order)
            for i in 0..10 {
                let (ch, style) = if i < (10 - hp_max) {
                    // Beyond max HP: red
                    ('█', Style::default().fg(THEME.palette.foe))
                } else if i < (10 - hp) {
                    // Lost HP but within max: timeline color
                    (
                        '█',
                        Style::default().fg(Color::from_hsl(THEME.palette.timeline_tail)),
                    )
                } else {
                    // Current HP: green
                    ('█', Style::default().fg(THEME.palette.ally))
                };
                buf.set_string(second.x + 4 + i as u16, second.y, ch.to_string(), style);
            }

            // Render Focus text
            Line::from(format!("Focus:{:2}", info.focus))
                .right_aligned()
                .render(second, buf);
        } else {
            Line::from("-")
                .style(THEME.palette.ui_disabled)
                .right_aligned()
                .render(second, buf);
        }
    }
}

pub struct TouchSenseWidget<'a> {
    pub sense: bool,
    pub info: Option<&'a losig_core::sense::TouchInfo>,
    pub selected: bool,
}

impl<'a> Widget for TouchSenseWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let status = self.info.map(|info| match (info.foes, info.orb) {
            (0, false) => Line::from("Nothing nearby"),
            (1, false) => Line::from("I touched something!").style(THEME.palette.foe),
            (n, false) => Line::from(format!("I touched {n} things!")).style(THEME.palette.foe),
            (0, true) => Line::from("The orb is nearby!").style(THEME.palette.important),
            (1, true) => Line::from(vec![
                Span::from("I touched something...").style(THEME.palette.foe),
                Span::from(" And the orb!").style(THEME.palette.important),
            ]),
            (n, true) => Line::from(vec![
                Span::from(format!("I touched {n} things...")).style(THEME.palette.foe),
                Span::from(" And the orb!").style(THEME.palette.important),
            ]),
        });

        let indicator = if self.sense { "(+)" } else { "(-)" };

        SenseWidget {
            label: "Touch",
            indicator,
            status,
            selected: self.selected,
            active: self.sense,
        }
        .render(area, buf);
    }
}

pub struct HearingSenseWidget<'a> {
    pub sense: BoundedU8<0, 5>,
    pub info: Option<&'a losig_core::sense::HearingInfo>,
    pub selected: bool,
}

impl<'a> Widget for HearingSenseWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let status = self.info.map(|str| match str.range {
            Some(range) => match range.get() {
                1 => Line::from("The orb is buzzing nearby!"),
                2 => Line::from("The orb is buzzing somewhat close"),
                3 => Line::from("The orb is buzzing"),
                4 => Line::from("The orb is buzzing distantly"),
                5 => Line::from("The orb is buzzing in the far distance"),
                _ => unreachable!(),
            }
            .style(THEME.palette.important),
            None => Line::from("Nothing"),
        });

        let indicator = format!("({})", self.sense);

        SenseWidget {
            label: "Hearing",
            indicator: indicator.as_str(),
            status,
            selected: self.selected,
            active: !self.sense.is_min(),
        }
        .render(area, buf);
    }
}

pub struct SightSenseWidget<'a> {
    pub sense: BoundedU8<0, 10>,
    pub info: Option<&'a losig_core::sense::SightInfo>,
    pub selected: bool,
}

impl<'a> Widget for SightSenseWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let status = self.info.map(|_| Line::from("I see stuff"));
        let indicator = format!("({})", self.sense);

        SenseWidget {
            label: "Sight",
            indicator: indicator.as_str(),
            status,
            selected: self.selected,
            active: !self.sense.is_min(),
        }
        .render(area, buf);
    }
}

pub struct SensesWidget<'a> {
    pub senses: Senses,
    pub info: Option<&'a SensesInfo>,
    pub selection: usize,
    pub max_sense: usize,
}

impl<'a> Widget for SensesWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let rows = Layout::vertical(vec![
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(area);
        let mut row_index = 0;

        SelfSenseWidget {
            sense: self.senses.selfs,
            info: self.info.and_then(|i| i.selfi.as_ref()),
            selected: self.selection == row_index,
        }
        .render(rows[row_index], buf);
        row_index += 1;

        if self.max_sense < 1 {
            return;
        }

        TouchSenseWidget {
            sense: self.senses.touch,
            info: self.info.and_then(|i| i.touch.as_ref()),
            selected: self.selection == row_index,
        }
        .render(rows[row_index], buf);
        row_index += 1;

        if self.max_sense < 2 {
            return;
        }

        HearingSenseWidget {
            sense: self.senses.hearing,
            info: self.info.and_then(|i| i.hearing.as_ref()),
            selected: self.selection == row_index,
        }
        .render(rows[row_index], buf);
        row_index += 1;

        if self.max_sense < 3 {
            return;
        }

        SightSenseWidget {
            sense: self.senses.sight,
            info: self.info.and_then(|i| i.sight.as_ref()),
            selected: self.selection == row_index,
        }
        .render(rows[row_index], buf);
    }
}
