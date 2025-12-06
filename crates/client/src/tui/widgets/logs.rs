use losig_core::{
    events::{GameEvent, Target},
    types::FoeType,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    text::Line,
    widgets::Widget,
};

use crate::{
    logs::{ClientLog, GameLog, LogEvent},
    tui::{THEME, theme::FoeTypeRender},
};

pub struct LogsWidget<'a> {
    pub logs: &'a [GameLog],
    pub current_turn: u64,
}

impl<'a> Widget for LogsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let max_lines = area.height as usize;

        let start = self.logs.len().saturating_sub(max_lines);
        let logs_to_show = &self.logs[start..];

        for (i, log) in logs_to_show.iter().enumerate() {
            let y = area.y + i as u16;
            let line_area = Rect {
                x: area.x,
                y,
                width: area.width,
                height: 1,
            };
            format_log(&log.log).render(line_area, buf);
        }
    }
}

fn format_log(log: &LogEvent) -> Line<'_> {
    match log {
        LogEvent::Client(client_log) => format_client_log(client_log),
        LogEvent::Server(event) => format_game_event(event.event()),
    }
}

fn format_client_log(log: &ClientLog) -> Line<'_> {
    match log {
        ClientLog::Help => Line::from("Press '?' for help"),
    }
}

fn format_game_event(event: &GameEvent) -> Line<'_> {
    let (line, style) = match event {
        GameEvent::Attack { subject, source } => (
            format!(
                "{} attacked {}.",
                format_target(source),
                format_target(subject)
            ),
            None,
        ),
        GameEvent::Fumble(target) => (
            format!("{} fumbled his attack.", format_target(target)),
            Some(THEME.palette.log_averted),
        ),
        GameEvent::Kill { subject, source } => (
            format!(
                "{} was killed by {}.",
                format_target(subject),
                format_target(source)
            ),
            None,
        ),
        GameEvent::ParadoxDeath(foe_type) => (
            format!("{} died from a heart attack.", format_foe_type(*foe_type)),
            Some(THEME.palette.log_paradox),
        ),
        GameEvent::ParadoxTeleport(foe_type) => (
            format!("{} was teleported.", format_foe_type(*foe_type)),
            Some(THEME.palette.log_paradox),
        ),
        GameEvent::OrbSeen => (
            "The orb glitches as you gaze upon it.".to_string(),
            Some(THEME.palette.important),
        ),
        GameEvent::OrbTaken(Target::You) => (
            "The world fades as you lay your hands on the orb.".to_string(),
            Some(THEME.palette.important),
        ),
        GameEvent::OrbTaken(other) => (
            format!("{} lays his hands on the orb.", format_target(other)),
            Some(THEME.palette.important),
        ),
        GameEvent::AvatarFadedOut(target) => (
            format!(
                "{} fades out as you move forward in time.",
                format_target(target)
            ),
            Some(Color::from_hsl(THEME.palette.timeline_tail)),
        ),
    };

    let mut result = Line::from(capitalize_first(&line));
    if let Some(color) = style {
        result = result.fg(color);
    }
    if matches!(event, GameEvent::OrbSeen) {
        result = result.bold();
    }
    result
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn format_target(target: &Target) -> String {
    match target {
        Target::Foe(foe_type) => format_foe_type(*foe_type),
        Target::You => "you".to_string(),
        Target::Player(_id, name) => name.clone(),
        Target::DiscardedAvatar => "a discarded avatar".to_string(),
        Target::Unknown => "something".to_string(),
        Target::Avatar(_) => unreachable!("Avatar target cannot be sent by server."),
    }
}

fn format_foe_type(ftype: FoeType) -> String {
    format!("the {}", ftype.label())
}
