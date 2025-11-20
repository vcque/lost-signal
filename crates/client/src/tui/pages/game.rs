use losig_core::{
    sense::{SenseInfo, Senses},
    types::{Action, Direction, Offset, Tile},
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

use crate::{
    logs::{ClientLog, GameLog},
    tui::{
        GameState, THEME, component::Component, state::TuiState, utils::center,
        widgets::block_wrap::BlockWrap,
    },
    tui_adapter::{Event, KeyCode},
    world::WorldView,
};

pub struct GamePage {}

impl GamePage {
    fn layout(area: Rect) -> [Rect; 3] {
        let [main, senses] =
            Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
                .areas(area);

        let [world, log] =
            Layout::vertical([Constraint::Percentage(80), Constraint::Percentage(20)]).areas(main);

        [world, log, senses]
    }
}

impl Component for GamePage {
    type State = TuiState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        {
            {
                let [world_a, log_a, _senses_a] = Self::layout(area);
                let world = &state.external.world.lock().unwrap();
                let world_widget = WorldViewWidget { world };

                let world_title = Line::from(Span::raw(format!(
                    "World - stage {} - turn {}",
                    world.stage + 1,
                    world.turn
                )));

                Block::default()
                    .borders(Borders::ALL)
                    .title(world_title)
                    .wrap(world_widget)
                    .render(world_a, buf);

                let logs_widget = LogsWidget {
                    logs: world.logs.logs(),
                };
                Block::default()
                    .borders(Borders::ALL)
                    .title("Game Log")
                    .wrap(logs_widget)
                    .render(log_a, buf);

                let game_state = &mut state.game;
                let senses_widget = SensesWidget {
                    senses: game_state.senses.clone(),
                    info: world.last_info(),
                    selection: game_state.sense_selection,
                };

                let cost = game_state.senses.signal_cost();
                let cost_style = if cost > world.current_state.signal {
                    Style::default().bold().bg(Color::Red)
                } else {
                    Style::default()
                };

                let title = Line::from(format!(
                    "Senses - cost: {} / {}",
                    cost,
                    world.current_state().signal
                ))
                .alignment(ratatui::layout::Alignment::Center);

                let senses_wigdet = Block::default()
                    .borders(Borders::TOP)
                    .title(title)
                    .border_style(cost_style)
                    .wrap(senses_widget);
                senses_wigdet.render(_senses_a, buf);
            }

            let winner = state.external.world.lock().unwrap().winner;
            if winner {
                YouWinWidget {}.render(area, buf, state);
            }
        }

        if state.game.show_help {
            HelpWidget {}.render(area, buf, &mut state.game);
        }
    }

    fn on_event(self, event: &Event, state: &mut Self::State) -> bool {
        // If help is visible, let HelpWidget handle the event
        if state.game.show_help {
            return HelpWidget {}.on_event(event, &mut state.game);
        }

        let Event::Key(key) = event else {
            return false;
        };

        {
            let winner: bool;
            {
                winner = state.external.world.lock().unwrap().winner;
            }
            if winner && (YouWinWidget {}).on_event(event, state) {
                return true;
            }
        }

        let game_state = &mut state.game;
        if key.modifiers.shift {
            let mut consumed = true;
            match key.code {
                KeyCode::Up | KeyCode::Char('8') => {
                    if game_state.sense_selection > 0 {
                        game_state.sense_selection -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('2') => {
                    if game_state.sense_selection < 4 {
                        game_state.sense_selection += 1;
                    }
                }
                KeyCode::Right | KeyCode::Char('6') => {
                    game_state.selected_sense_mut().incr();
                }
                KeyCode::Left | KeyCode::Char('4') => {
                    game_state.selected_sense_mut().decr();
                }
                _ => {
                    consumed = false;
                }
            }
            if consumed {
                return true;
            }
        }

        let action = match key.code {
            KeyCode::Up | KeyCode::Char('8') | KeyCode::Char('k') => {
                Some(Action::Move(Direction::Up))
            }
            KeyCode::Down | KeyCode::Char('2') | KeyCode::Char('j') => {
                Some(Action::Move(Direction::Down))
            }
            KeyCode::Left | KeyCode::Char('4') | KeyCode::Char('h') => {
                Some(Action::Move(Direction::Left))
            }
            KeyCode::Right | KeyCode::Char('6') | KeyCode::Char('l') => {
                Some(Action::Move(Direction::Right))
            }
            KeyCode::Char('7') | KeyCode::Char('y') => Some(Action::Move(Direction::UpLeft)),
            KeyCode::Char('9') | KeyCode::Char('u') => Some(Action::Move(Direction::UpRight)),
            KeyCode::Char('1') | KeyCode::Char('b') => Some(Action::Move(Direction::DownLeft)),
            KeyCode::Char('3') | KeyCode::Char('n') => Some(Action::Move(Direction::DownRight)),
            KeyCode::Char('5') | KeyCode::Char(' ') => Some(Action::Wait),
            KeyCode::Char('r') => Some(Action::Spawn),
            KeyCode::Char('?') => {
                game_state.show_help = true;
                return true;
            }
            _ => None,
        };
        if let Some(action) = action {
            state.external.act(action, game_state.senses.clone());
            return true;
        }
        false
    }
}

const SPAWN_STYLE: &Style = &Style::new().fg(Color::LightYellow);
const PYLON_STYLE: &Style = &Style::new().bg(Color::Gray).fg(Color::LightBlue);
const WALL_STYLE: &Style = &Style::new().bg(Color::Gray);
const DEFAULT_STYLE: &Style = &Style::new();

fn render_tile(tile: Tile) -> (char, &'static Style) {
    match tile {
        Tile::Spawn => ('S', SPAWN_STYLE),
        Tile::Wall => (' ', WALL_STYLE),
        Tile::Unknown => (' ', DEFAULT_STYLE),
        Tile::Empty => ('.', DEFAULT_STYLE),
        Tile::Pylon => ('|', PYLON_STYLE),
    }
}

struct WorldViewWidget<'a> {
    world: &'a WorldView,
}

impl<'a> Widget for WorldViewWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let w = self.world;
        let center_x = area.width as isize / 2;
        let center_y = area.height as isize / 2;

        for x in 0..area.width {
            for y in 0..area.height {
                let tile = w.current_state().tile_from_viewer(Offset {
                    x: x as isize - center_x,
                    y: y as isize - center_y,
                });

                let (ch, style) = render_tile(tile);
                buf.set_string(area.x + x, area.y + y, ch.to_string(), *style);
            }
        }

        let mut style = Style::default();
        if self.world.current_state().broken {
            style = style.red();
        }

        buf.set_string(
            area.x + center_x as u16,
            area.y + center_y as u16,
            "@",
            style,
        );
    }
}

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
            (true, _) => THEME.styles.selection,
            (_, true) => THEME.styles.active,
            _ => THEME.styles.inactive,
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

        let status = status.unwrap_or(Line::from("-").style(THEME.styles.inactive));
        status.right_aligned().render(second, buf);
    }
}

struct SensesWidget {
    senses: Senses,
    info: SenseInfo,
    selection: usize,
}

impl Widget for SensesWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let rows = Layout::vertical([Constraint::Length(2); 4]).split(area);
        let mut row_index = 0;

        let sense = self.senses.selfs;
        let status = self.info.selfi.map(|selfi| {
            if selfi.broken {
                Line::from("Broken").style(THEME.styles.danger)
            } else {
                Line::from("Intact")
            }
        });

        let indicator = if sense.is_some() { "(+)" } else { "(-)" };

        SenseWidget {
            label: "Self",
            indicator,
            status,
            selected: self.selection == row_index,
            active: sense.is_some(),
        }
        .render(rows[row_index], buf);
        row_index += 1;

        let sense = self.senses.terrain;
        let status = self
            .info
            .terrain
            .map(|t| Line::from(format!("{} tiles", t.tiles.len())));
        let indicator = match sense {
            Some(t) => format!("({})", t.radius),
            None => "(-)".to_owned(),
        };

        SenseWidget {
            label: "Terrain",
            indicator: indicator.as_str(),
            status,
            selected: self.selection == row_index,
            active: sense.is_some(),
        }
        .render(rows[row_index], buf);
        row_index += 1;

        let sense = self.senses.danger;
        let status = self.info.danger.map(|prox| match prox.count {
            0 => Line::from("Nothing"),
            1 => Line::from("There is a threat nearby").style(THEME.styles.danger),
            n => Line::from(format!("There are {} threats nearby", n)).style(THEME.styles.danger),
        });

        let indicator = match sense {
            Some(s) => format!("({})", s.radius),
            None => "(-)".to_owned(),
        };

        SenseWidget {
            label: "Danger",
            indicator: indicator.as_str(),
            status,
            selected: self.selection == row_index,
            active: sense.is_some(),
        }
        .render(rows[row_index], buf);
        row_index += 1;

        let sense = self.senses.orb;
        let status = self.info.orb.map(|orb| {
            if orb.detected {
                Line::from("I can feel it").style(THEME.styles.signal)
            } else {
                Line::from("Nothing")
            }
        });

        let indicator = match sense {
            Some(s) => match s.level {
                losig_core::sense::SenseLevel::Minimum => "(+)",
                losig_core::sense::SenseLevel::Low => "(++)",
                losig_core::sense::SenseLevel::Medium => "(+++)",
                losig_core::sense::SenseLevel::High => "(++++)",
                losig_core::sense::SenseLevel::Maximum => "(+++++)",
            },
            None => "(-)",
        };

        SenseWidget {
            label: "Orb",
            indicator,
            status,
            selected: self.selection == row_index,
            active: sense.is_some(),
        }
        .render(rows[row_index], buf);
    }
}

struct YouWinWidget {}

impl Component for YouWinWidget {
    type State = TuiState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let popup_width = 50;
        let popup_height = 10;

        let popup_area = center(
            area,
            Constraint::Length(popup_width),
            Constraint::Length(popup_height),
        );

        // Clear background
        for x in popup_area.x..popup_area.x + popup_area.width {
            for y in popup_area.y..popup_area.y + popup_area.height {
                buf.set_string(x, y, " ", Style::default().bg(Color::Black));
            }
        }

        let block = Block::default()
            .title("🎉 Victory! 🎉")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::Green));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        if state.you_win.sent {
            // Show submission confirmation
            let lines = [
                "Your score has been submitted.",
                "",
                "Thank you for playing!",
            ];

            for (i, line) in lines.iter().enumerate() {
                let y = inner.y + 2 + i as u16;
                let x = inner.x + (inner.width.saturating_sub(line.len() as u16)) / 2;
                let style = if i == 0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Gray)
                };
                buf.set_string(x, y, line, style);
            }
        } else {
            // Show name input form
            let lines = [
                "Enter your name for the leaderboard:",
                "",
                &format!("> {}_", state.you_win.name),
                "",
                "(Max 8 characters, press Enter to submit)",
            ];

            for (i, line) in lines.iter().enumerate() {
                let y = inner.y + 1 + i as u16;
                let x = inner.x + (inner.width.saturating_sub(line.len() as u16)) / 2;
                let style = match i {
                    0 => Style::default().fg(Color::White),
                    2 => Style::default().fg(Color::Yellow),
                    4 | 5 => Style::default().fg(Color::Gray),
                    _ => Style::default().fg(Color::White),
                };
                buf.set_string(x, y, line, style);
            }
        }
    }
    fn on_event(self, event: &Event, state: &mut Self::State) -> bool {
        // Handle YouWin events
        let you_win = &mut state.you_win;

        if you_win.sent {
            // Already sent, any key closes win screen
            state.you_win.open = false;
            return true;
        }

        let Event::Key(event) = event else {
            return false;
        };

        match event.code {
            KeyCode::Enter => {
                if !you_win.name.is_empty() {
                    state.external.submit_leaderboard(you_win.name.clone());
                    you_win.sent = true;
                }
            }
            KeyCode::Backspace => {
                you_win.name.pop();
            }
            KeyCode::Char(c) if you_win.name.len() < 8 => {
                you_win.name.push(c);
            }
            KeyCode::Esc => {
                you_win.open = false;
            }
            _ => {}
        };

        true
    }
}

struct HelpWidget {}

impl Component for HelpWidget {
    type State = GameState;

    fn on_event(self, event: &Event, state: &mut Self::State) -> bool {
        let Event::Key(key) = event else {
            return true; // Consume all non-key events when help is visible
        };

        match key.code {
            KeyCode::Char('?') | KeyCode::Esc => {
                state.show_help = false;
                true
            }
            _ => true, // Consume all other key events when help is visible
        }
    }

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        self.render_widget(area, buf);
    }
}

impl HelpWidget {
    fn render_widget(self, area: Rect, buf: &mut Buffer) {
        let popup_width = 80;
        let popup_height = 22;

        let popup_area = center(
            area,
            Constraint::Length(popup_width),
            Constraint::Length(popup_height),
        );

        // Clear the popup area with a background
        for x in popup_area.x..popup_area.x + popup_area.width {
            for y in popup_area.y..popup_area.y + popup_area.height {
                buf.set_string(x, y, " ", Style::default().bg(Color::Black));
            }
        }

        let block = Block::default()
            .title("Help - Press '?' or 'ESC' to close")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::White));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        let header_style = Style::default().fg(Color::Yellow).bold();
        let help_text = vec![
            Line::from(Span::styled("CONTROLS", header_style)),
            Line::from("Movement: Arrow Keys, Vi keys (hjklyubn), or Numpad (8246 + 7913)"),
            Line::from("Wait: 5 or Space |  Respawn: r  |  Help: ?"),
            Line::from("Sense Controls (Shift + Key): Up/Down=Select, Left/Right=Adjust"),
            Line::from(""),
            Line::from(Span::styled("SENSES", header_style)),
            Line::from("Self: Monitor your integrity"),
            Line::from("Terrain: See nearby tiles (radius)"),
            Line::from("Danger: Detect threats (radius)"),
            Line::from("Orb: Detect your goal"),
            Line::from(""),
            Line::from(Span::styled("SIGNAL", header_style)),
            Line::from("Each sense costs signal points per turn"),
            Line::from("Pylons restore your signal"),
            Line::from("Manage your signal budget carefully"),
            Line::from(""),
            Line::from(Span::styled("GOAL", header_style)),
            Line::from("Find and reach the orb to win the game"),
            Line::from("Use your senses to navigate the world"),
        ];

        for (i, line) in help_text.iter().enumerate() {
            if i < inner.height as usize {
                line.render(
                    Rect {
                        x: inner.x,
                        y: inner.y + i as u16,
                        width: inner.width,
                        height: 1,
                    },
                    buf,
                );
            }
        }
    }
}

struct LogsWidget<'a> {
    logs: &'a [GameLog],
}

impl<'a> Widget for LogsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let max_lines = area.height as usize;
        let logs_to_show = self.logs.iter().rev().take(max_lines);

        for (i, log) in logs_to_show.enumerate() {
            if i >= max_lines {
                break;
            }

            let y = area.y + i as u16;
            let (message, message_style) = format_log(log.log);
            let turn_text = format!("turn {}: ", log.turn);

            // Render turn text with default style
            buf.set_string(area.x, y, &turn_text, Style::default());

            // Render message with styled text
            buf.set_string(area.x + turn_text.len() as u16, y, message, message_style);
        }
    }
}

fn format_log(log: ClientLog) -> (&'static str, Style) {
    match log {
        ClientLog::Help => ("Press '?' for help", Style::default().fg(Color::Cyan)),
        ClientLog::NextStage => ("I'm making progress.", Style::default().fg(Color::Green)),
        ClientLog::Lost => ("I am lost...", Style::default().fg(Color::Red)),
        ClientLog::Win => ("I won!", Style::default().fg(Color::Yellow).bold()),
    }
}
