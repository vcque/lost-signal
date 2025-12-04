use std::cmp::Ordering;

use bounded_integer::BoundedU8;
use itertools::Itertools;
use losig_core::sense::{SenseStrength, Senses, SensesInfo, SightInfo, SightedAllyStatus};
use losig_core::types::{FOCUS_MAX, FoeType, HP_MAX, StageTurn};
use ratatui::layout::Spacing;
use ratatui::widgets::Paragraph;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::tui::{FoeTypeRender, THEME};

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
        let layout = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
        let [header, content] = layout.areas(area);

        // Render header
        let indicator = if self.sense { "(+)" } else { "(-)" };
        render_sense_header(header, buf, "Touch", indicator, self.selected, self.sense);

        let lines: Vec<Line> = match self.info {
            Some(info) => {
                let mut lines = vec![];

                if info.orb {
                    lines.push(Line::from(vec![
                        Span::from("o").style(THEME.palette.important),
                        Span::from(": the orb"),
                    ]));
                }

                if !info.foes.is_empty() {
                    lines.push(Line::from(vec![
                        Span::from("?").style(THEME.palette.foe),
                        Span::from(format!(
                            ": {} foe{}",
                            info.foes.len(),
                            if info.foes.len() == 1 { "" } else { "s" }
                        )),
                    ]));
                }

                if info.traps > 0 {
                    lines.push(Line::from(vec![
                        Span::from("¤").style(THEME.palette.trap),
                        Span::from(format!(
                            ": {} trap{}",
                            info.traps,
                            if info.traps == 1 { "" } else { "s" }
                        )),
                    ]));
                }

                if lines.is_empty() {
                    lines.push(Line::from("Nothing nearby"));
                }

                lines
            }
            None => vec![
                Line::from("-")
                    .style(THEME.palette.ui_disabled)
                    .right_aligned(),
            ],
        };

        Paragraph::new(lines).render(content, buf);
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
    pub stage_turn: StageTurn,
    pub info: Option<&'a SightInfo>,
    pub selected: bool,
}

impl<'a> Widget for SightSenseWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
        let [header, content] = layout.areas(area);

        // Render header
        render_sense_header(
            header,
            buf,
            "Sight",
            &format!("({})", self.sense),
            self.selected,
            !self.sense.is_min(),
        );

        let lines: Vec<Line> = match self.info {
            Some(info) => {
                let lines = to_widget_lines(info, content.height as usize);
                lines
                    .into_iter()
                    .map(|l| render_sight_line(l, self.stage_turn))
                    .collect()
            }
            None => vec![
                Line::from("-")
                    .style(THEME.palette.ui_disabled)
                    .right_aligned(),
            ],
        };

        Paragraph::new(lines).render(content, buf);
    }
}

fn render_sight_line<'a>(l: SightWidgetLine, stage_turn: u64) -> Line<'a> {
    match l {
        SightWidgetLine::Orb => Line::from(vec![
            Span::from("o").style(THEME.palette.important),
            Span::from(": the orb"),
        ]),
        SightWidgetLine::Foe(foe_type, count) => Line::from(vec![
            Span::from(foe_type.grapheme()).style(THEME.palette.foe),
            Span::from(format!(": {} {}", count, foe_type.label())),
        ]),
        SightWidgetLine::Ally(turn, name) => {
            let diff = turn.abs_diff(stage_turn);
            let (color, label) = match turn.cmp(&stage_turn) {
                Ordering::Greater => (THEME.palette.ally_leading, format!("{diff} turns ahead")),
                Ordering::Equal => (THEME.palette.ally_sync, "on the same turn".to_owned()),
                Ordering::Less => (THEME.palette.ally_trailing, format!("{diff} turns behind")),
            };

            Line::from(vec![
                Span::from("@").style(color),
                Span::from(format!(": {name} ({label})")),
            ])
        }
    }
}

fn to_widget_lines(info: &SightInfo, max_lines: usize) -> Vec<SightWidgetLine> {
    let mut lines = vec![];

    if info.orb.is_some() {
        lines.push(SightWidgetLine::Orb);
    }

    let foe_lines = info
        .foes
        .iter()
        .filter(|f| f.alive)
        .map(|f| f.foe_type)
        .counts()
        .into_iter()
        .map(|(foe_type, count)| SightWidgetLine::Foe(foe_type, count))
        .collect::<Vec<_>>();

    lines.extend(foe_lines);

    let ally_lines = info
        .allies
        .iter()
        .filter(|al| al.alive)
        .filter_map(|al| match &al.status {
            SightedAllyStatus::Controlled { turn, name } => Some((*turn, name.clone())),
            SightedAllyStatus::Discarded => None,
        })
        .sorted()
        .map(|(turn, name)| SightWidgetLine::Ally(turn, name))
        .collect_vec();

    lines.extend(ally_lines);

    lines.truncate(max_lines);
    lines
}

#[derive(Eq, PartialEq)]
enum SightWidgetLine {
    Orb,
    Foe(FoeType, usize),
    Ally(StageTurn, String),
}

pub struct SensesWidget<'a> {
    pub stage_turn: StageTurn,
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
            Constraint::Min(2),
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
            stage_turn: self.stage_turn,
            sense: self.senses.sight,
            info: self.info.and_then(|i| i.sight.as_ref()),
            selected: self.selection == row_index,
        }
        .render(rows[row_index], buf);
    }
}
