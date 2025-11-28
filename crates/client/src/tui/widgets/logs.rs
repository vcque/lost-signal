use losig_core::types::{GameLogEvent, Target};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Widget,
};

use crate::{
    logs::{ClientLog, GameLog, LOG_RECENT_THRESHOLD, LogEvent},
    tui::THEME,
};

pub struct LogsWidget<'a> {
    pub logs: &'a [GameLog],
    pub current_turn: u64,
}

impl<'a> Widget for LogsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let max_lines = area.height as usize;

        let logs_to_show: Vec<_> = self.logs.iter().rev().take(max_lines).collect();

        for (i, log) in logs_to_show.iter().enumerate() {
            if i >= max_lines {
                break;
            }

            let y = area.y + i as u16;

            // Determine the turn to display and whether the log is averted or out-of-time
            let (is_averted, is_revision, is_recent) = match &log.log {
                LogEvent::Client(_) => (false, false, false),
                LogEvent::Server {
                    received, averted, ..
                } => {
                    let averted = averted.is_some();
                    let revision = !averted && received.abs_diff(log.turn) > 1;
                    let recent = received + LOG_RECENT_THRESHOLD > self.current_turn;
                    (averted, revision, recent)
                }
            };

            let (message, mut message_style) = format_log(&log.log);
            if is_averted {
                // I'd prefer the possibility to overload style from the line but... here we are.
                message_style = Default::default();
            }

            let mut turn_style = Style::default();
            if is_recent {
                // Apply crossed-out style for averted logs
                if is_revision {
                    // Out-of-time logs: cyan background, black foreground for turn text
                    turn_style = turn_style
                        .bg(THEME.palette.log_revision_bg)
                        .fg(THEME.palette.log_revision_fg);
                }
            }

            let turn_span = Span::from(format!("turn {}", log.turn)).style(turn_style);
            let log_span = Span::from(message).style(message_style);

            let mut line = Line::from(vec![turn_span, Span::from(": "), log_span]);
            if is_averted {
                let averted_style = Style::from(THEME.palette.log_averted).crossed_out();
                line = line.style(averted_style);
            }

            let line_area = Rect {
                x: area.x,
                y,
                width: line.width() as u16,
                height: 1,
            };
            line.render(line_area, buf);
        }
    }
}

fn format_log(log: &LogEvent) -> (&'static str, Style) {
    match log {
        LogEvent::Client(ClientLog::Help) => (
            "Press '?' for help",
            Style::default().fg(THEME.palette.log_info),
        ),
        LogEvent::Server { event, .. } => match event {
            GameLogEvent::Attack { from, to } => match (from, to) {
                (Target::You, Target::Foe(_)) => {
                    ("You attacked!", Style::default().fg(THEME.palette.log_info))
                }
                (Target::Foe(_), Target::You) => (
                    "You were attacked!",
                    Style::default().fg(THEME.palette.log_grave),
                ),
                _ => ("Attack!", Style::default()),
            },
            GameLogEvent::StageUp(_) => ("Stage up!", Style::default().fg(THEME.palette.important)),
            GameLogEvent::Defeated { from, to } => match (from, to) {
                (Target::You, Target::Foe(_)) => (
                    "You defeated an enemy!",
                    Style::default().fg(THEME.palette.log_info),
                ),
                (Target::Foe(_), Target::You) => (
                    "You were defeated!",
                    Style::default().fg(THEME.palette.log_grave),
                ),
                _ => ("Defeated!", Style::default()),
            },
            GameLogEvent::OrbSeen => (
                "The orb glitches as you gaze upon it!",
                Style::default().fg(THEME.palette.important),
            ),
            GameLogEvent::Spawn => ("Respawned", Style::default().fg(THEME.palette.ui_text)),
        },
    }
}
