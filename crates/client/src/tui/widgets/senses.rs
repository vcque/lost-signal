use bounded_integer::BoundedU8;
use losig_core::sense::{SenseStrength, Senses, SensesInfo, SightInfo};
use losig_core::types::{FOCUS_MAX, HP_MAX};
use ratatui::layout::Spacing;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::tui::THEME;

/// Renders the common header line for a sense widget (label, indicator, selection styling)
fn render_sense_header(
    area: Rect,
    buf: &mut Buffer,
    label: &str,
    indicator: &str,
    selected: bool,
    active: bool,
) {
    let style = match (selected, active) {
        (true, _) => THEME.palette.ui_selected,
        (_, true) => THEME.palette.ui_highlight,
        _ => THEME.palette.ui_disabled,
    };

    buf.set_string(area.x, area.y, ".".repeat(area.width as usize), style);
    Line::from(label).style(style).render(area, buf);
    Line::from(indicator)
        .style(style)
        .right_aligned()
        .render(area, buf);
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

        // Render header
        let indicator = if self.sense { "(+)" } else { "(-)" };
        render_sense_header(first, buf, "Self", indicator, self.selected, self.sense);

        // Render content (HP gauge + Focus gauge)
        if let Some(info) = self.info {
            // Split the second line into two equal halves
            let halves =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .spacing(Spacing::Space(2))
                    .split(second);
            let [hp_area, fp_area] = halves.as_ref() else {
                return;
            };

            // Render HP gauge in left half
            let hp = info.hp.min(HP_MAX);
            let hp_max = info.hp_max.min(HP_MAX);

            buf.set_string(hp_area.x, hp_area.y, "HP: ", Style::default());

            // Calculate number of blocks based on available width (subtract 4 for "HP: " label)
            let hp_blocks = hp_area.width.saturating_sub(4) as usize;

            // Render HP gauge using ratio
            for i in 0..hp_blocks {
                let threshold = ((i + 1) as f32 / hp_blocks as f32 * HP_MAX as f32) as u8;
                let (ch, style) = if hp >= threshold {
                    // Current HP: green
                    ('█', Style::default().fg(THEME.palette.ui_hp))
                } else if hp_max >= threshold {
                    // Lost HP but within max: timeline color
                    (
                        '█',
                        Style::default().fg(Color::from_hsl(THEME.palette.timeline_tail)),
                    )
                } else {
                    // Beyond max HP: red
                    ('█', Style::default().fg(THEME.palette.ui_bar_empty))
                };
                buf.set_string(hp_area.x + 4 + i as u16, hp_area.y, ch.to_string(), style);
            }

            // Render FP gauge in right half
            let focus = info.focus.min(FOCUS_MAX);

            buf.set_string(fp_area.x, fp_area.y, "FP: ", Style::default());

            // Calculate number of blocks based on available width (subtract 4 for "FP: " label)
            let fp_blocks = fp_area.width.saturating_sub(4) as usize;

            // Render FP gauge using ratio
            for i in 0..fp_blocks {
                let threshold = ((i + 1) as f32 / fp_blocks as f32 * FOCUS_MAX as f32) as u8;
                let (ch, style) = if focus >= threshold {
                    ('█', Style::default().fg(THEME.palette.ui_focus))
                } else {
                    ('█', Style::default().fg(THEME.palette.ui_bar_empty))
                };
                buf.set_string(fp_area.x + 4 + i as u16, fp_area.y, ch.to_string(), style);
            }
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
        let layout = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]);
        let [first, second] = layout.areas(area);

        // Render header
        let indicator = if self.sense { "(+)" } else { "(-)" };
        render_sense_header(first, buf, "Touch", indicator, self.selected, self.sense);

        // Render content
        let status = self
            .info
            .map(|info| match (info.foes, info.orb) {
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
            })
            .unwrap_or(Line::from("-").style(THEME.palette.ui_disabled));

        status.right_aligned().render(second, buf);
    }
}

pub struct HearingSenseWidget<'a> {
    pub sense: BoundedU8<0, 5>,
    pub info: Option<&'a losig_core::sense::HearingInfo>,
    pub selected: bool,
}

impl<'a> Widget for HearingSenseWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]);
        let [first, second] = layout.areas(area);

        // Render header
        render_sense_header(
            first,
            buf,
            "Hearing",
            &format!("({})", self.sense),
            self.selected,
            !self.sense.is_min(),
        );

        // Render content
        let status = self
            .info
            .map(|str| match str.range {
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
            })
            .unwrap_or(Line::from("-").style(THEME.palette.ui_disabled));

        status.right_aligned().render(second, buf);
    }
}

pub struct SightSenseWidget<'a> {
    pub sense: BoundedU8<0, 10>,
    pub info: Option<&'a SightInfo>,
    pub selected: bool,
}

impl<'a> Widget for SightSenseWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]);
        let [first, second] = layout.areas(area);

        // Render header
        render_sense_header(
            first,
            buf,
            "Sight",
            &format!("({})", self.sense),
            self.selected,
            !self.sense.is_min(),
        );

        // Render content
        let status = self
            .info
            .map(|_| Line::from("I see stuff"))
            .unwrap_or(Line::from("-").style(THEME.palette.ui_disabled));

        status.right_aligned().render(second, buf);
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
