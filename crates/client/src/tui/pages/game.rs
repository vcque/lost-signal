use losig_core::{
    sense::{SenseStrength, Senses, SensesInfo},
    types::{Action, Direction, GameOver, Offset, Tile},
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

use crate::{
    logs::{ClientLog, GameLog},
    tui::{
        InputServices, RenderServices, THEME, YouWinState,
        state::TuiState,
        utils::center,
        widgets::{block_wrap::BlockWrap, help::HelpWidget},
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

    pub fn render(
        self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut TuiState,
        services: RenderServices,
    ) {
        let [world_a, log_a, _senses_a] = Self::layout(area);
        let world = &services.state.world;
        state.game.stage = 5; // TODO: fix this
        state.game.help.next_page(world.stage);

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
            max_sense: game_state.stage as usize,
        };

        let cost = game_state.senses.cost();
        let focus = world
            .last_info()
            .and_then(|info| info.selfi.as_ref())
            .map(|selfi| selfi.focus);
        let cost_style = if focus.is_some_and(|s| s < cost) {
            Style::default().bold().bg(THEME.palette.foe)
        } else {
            Style::default()
        };

        let focus_str = focus.map(|s| s.to_string()).unwrap_or("??".to_owned());

        let title = Line::from(format!(" Senses - cost: {} / {} ", cost, focus_str))
            .alignment(ratatui::layout::Alignment::Center);

        let senses_wigdet = Block::default()
            .borders(Borders::TOP)
            .title(title)
            .border_style(cost_style)
            .wrap(senses_widget);
        senses_wigdet.render(_senses_a, buf);

        if let Some(gameover) = &services.state.gameover {
            GameOverWidget {}.render(area, buf, gameover, &mut state.you_win);
        }

        if state.game.help.open {
            HelpWidget.render(area, buf, &state.game.help);
        }
    }

    pub fn on_event(
        self,
        event: &Event,
        state: &mut TuiState,
        mut services: InputServices,
    ) -> bool {
        // If help is visible, let HelpWidget handle the event
        if state.game.help.open {
            return HelpWidget.on_event(event, &mut state.game.help);
        }

        let Event::Key(key) = event else {
            return false;
        };

        if services.state.gameover.is_some()
            && (GameOverWidget {}).on_event(event, &mut state.you_win, &mut services)
        {
            return true;
        }

        let game_state = &mut state.game;
        if key.modifiers.shift {
            let mut consumed = true;
            match key.code {
                KeyCode::Up | KeyCode::Char('8') | KeyCode::Char('K') => {
                    game_state.sense_selection = game_state.sense_selection.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('2') | KeyCode::Char('J') => {
                    let max_sense = game_state.stage.min(3);
                    if game_state.sense_selection < max_sense as usize {
                        game_state.sense_selection += 1;
                    }
                }
                KeyCode::Right | KeyCode::Char('6') | KeyCode::Char('L') => {
                    game_state.incr_sense();
                }
                KeyCode::Left | KeyCode::Char('4') | KeyCode::Char('H') => {
                    game_state.decr_sense();
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
                game_state.help.open = true;
                return true;
            }
            _ => None,
        };
        if let Some(action) = action {
            services.act(action, game_state.senses.clone());
            return true;
        }
        false
    }
}

// Game tile styles are now inline to use THEME palette
const DEFAULT_STYLE: &Style = &Style::new();

fn render_tile(tile: Tile) -> (char, Style) {
    match tile {
        Tile::Spawn => ('^', Style::new().fg(THEME.palette.important)),
        Tile::Wall => (' ', Style::new().bg(THEME.palette.terrain)),
        Tile::Unknown => (' ', *DEFAULT_STYLE),
        Tile::Empty => ('.', Style::new().fg(THEME.palette.terrain)),
        Tile::Pylon => ('|', Style::new().fg(THEME.palette.important)),
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

        let last_info = w.last_info();
        for x in 0..area.width {
            for y in 0..area.height {
                let offset = Offset {
                    x: x as isize - center_x,
                    y: y as isize - center_y,
                };

                let tile = w.current_state().tile_from_viewer(offset);

                let in_fov = last_info
                    .as_ref()
                    .and_then(|i| i.sight.as_ref())
                    .map(|t| t.tiles.at_offset_from_center(offset))
                    .unwrap_or_default()
                    != Tile::Unknown;

                let (ch, style) = render_tile(tile);
                let style = if in_fov {
                    style
                } else if style.fg.is_some() {
                    style.fg(THEME.palette.terrain_unseen)
                } else if style.bg.is_some() {
                    style.bg(THEME.palette.terrain_unseen)
                } else {
                    style
                };
                buf.set_string(area.x + x, area.y + y, ch.to_string(), style);
            }
        }

        if let Some(sight) = last_info.and_then(|i| i.sight.as_ref()) {
            // Show the orb
            if let Some(ref offset) = sight.orb {
                let x = center_x + offset.x;
                let y = center_y + offset.y;

                buf.set_string(
                    area.x + x as u16,
                    area.y + y as u16,
                    "o",
                    THEME.palette.important,
                );
            }

            // Show the foes
            for foe in &sight.foes {
                let x = center_x + foe.x;
                let y = center_y + foe.y;

                buf.set_string(area.x + x as u16, area.y + y as u16, "¤", THEME.palette.foe);
            }

            // Show the foes
            for ally in &sight.allies {
                let x = center_x + ally.x;
                let y = center_y + ally.y;

                buf.set_string(
                    area.x + x as u16,
                    area.y + y as u16,
                    "@",
                    THEME.palette.ally,
                );
            }
        }

        let neigboring_foes = last_info
            .and_then(|i| i.touch.as_ref())
            .map(|it| it.foes)
            .unwrap_or_default();

        let has_sight = last_info.and_then(|i| i.sight.as_ref()).is_some();
        if !has_sight && neigboring_foes > 0 {
            buf.set_string(
                area.x + center_x as u16,
                area.y + center_y as u16,
                neigboring_foes.to_string(),
                THEME.palette.foe,
            );
        } else {
            buf.set_string(
                area.x + center_x as u16,
                area.y + center_y as u16,
                "@",
                THEME.palette.avatar,
            );
        }
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

struct SensesWidget<'a> {
    senses: Senses,
    info: Option<&'a SensesInfo>,
    selection: usize,
    max_sense: usize,
}

impl<'a> Widget for SensesWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let rows = Layout::vertical([Constraint::Length(2); 4]).split(area);
        let mut row_index = 0;

        let sense = self.senses.selfs;
        let info = self.info.and_then(|i| i.selfi.as_ref());
        let status = info.map(|i| {
            Line::from(vec![
                Span::from(format!("HP: {:2}", i.hp)),
                Span::from("   "),
                Span::from(format!("Focus: {:3}", i.focus)),
            ])
        });

        let indicator = if sense { "(+)" } else { "(-)" };

        SenseWidget {
            label: "Self",
            indicator,
            status,
            selected: self.selection == row_index,
            active: sense,
        }
        .render(rows[row_index], buf);
        row_index += 1;
        if self.max_sense < 1 {
            return;
        }

        let sense = self.senses.touch;
        let info = self.info.and_then(|i| i.touch.as_ref());
        let status = info.map(|info| match (info.foes, info.orb) {
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
        let indicator = if sense { "(+)" } else { "(-)" };

        SenseWidget {
            label: "Touch",
            indicator,
            status,
            selected: self.selection == row_index,
            active: sense,
        }
        .render(rows[row_index], buf);
        row_index += 1;

        if self.max_sense < 2 {
            return;
        }

        let sense = self.senses.hearing;
        let info = self.info.and_then(|i| i.hearing.as_ref());
        let status = info.map(|str| match str.range {
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
        let indicator = format!("({})", sense);

        SenseWidget {
            label: "Hearing",
            indicator: indicator.as_str(),
            status,
            selected: self.selection == row_index,
            active: !sense.is_min(),
        }
        .render(rows[row_index], buf);
        row_index += 1;
        if self.max_sense < 3 {
            return;
        }

        let sense = self.senses.sight;
        let info = self.info.and_then(|i| i.sight.as_ref());
        let status = info.map(|_| Line::from("I see stuff"));
        let indicator = format!("({})", sense);

        SenseWidget {
            label: "Sight",
            indicator: indicator.as_str(),
            status,
            selected: self.selection == row_index,
            active: !sense.is_min(),
        }
        .render(rows[row_index], buf);
    }
}

struct GameOverWidget {}

impl GameOverWidget {
    pub fn render(
        self,
        area: Rect,
        buf: &mut Buffer,
        gameover: &GameOver,
        state: &mut YouWinState,
    ) {
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
                buf.set_string(x, y, " ", Style::default());
            }
        }

        let (title, color) = if gameover.win {
            ("🎉 Victory! 🎉", THEME.palette.log_info)
        } else {
            ("💀 Game Over 💀", THEME.palette.log_grave)
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default().fg(color));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        if state.sent {
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
                    Style::default().fg(THEME.palette.important)
                } else {
                    Style::default().fg(THEME.palette.ui_disabled)
                };
                buf.set_string(x, y, line, style);
            }
        } else {
            // Show game stats and name input form
            let stage_line = format!("Stage: {}", gameover.stage);
            let turns_line = format!("Turns: {}", gameover.turns);
            let score_line = format!("Score: {}", gameover.score);
            let name_line = format!("> {}_", state.name);

            let stats_lines = [
                stage_line.as_str(),
                turns_line.as_str(),
                score_line.as_str(),
                "",
                "Enter your name for the leaderboard:",
                "",
                name_line.as_str(),
                "",
                "(Max 8 characters, press Enter to submit)",
            ];

            for (i, line) in stats_lines.iter().enumerate() {
                let y = inner.y + i as u16;
                let x = inner.x + (inner.width.saturating_sub(line.len() as u16)) / 2;
                let style = match i {
                    0..=2 => Style::default().fg(THEME.palette.important), // Stats
                    4 => Style::default().fg(THEME.palette.ui_text),       // Prompt
                    6 => Style::default().fg(THEME.palette.ui_text),       // Name input
                    8 => Style::default().fg(THEME.palette.ui_text),       // Instructions
                    _ => Style::default().fg(THEME.palette.ui_text),
                };
                buf.set_string(x, y, line, style);
            }
        }
    }
    fn on_event(
        self,
        event: &Event,
        state: &mut YouWinState,
        services: &mut InputServices,
    ) -> bool {
        // Handle YouWin events
        let you_win = state;

        if you_win.sent {
            // Already sent, any key closes win screen
            you_win.open = false;
            return true;
        }

        let Event::Key(event) = event else {
            return false;
        };

        match event.code {
            KeyCode::Enter => {
                if !you_win.name.is_empty() {
                    services.submit_leaderboard(you_win.name.clone());
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
        ClientLog::Help => (
            "Press '?' for help",
            Style::default().fg(THEME.palette.log_info),
        ),
        ClientLog::NextStage => (
            "I have reached a higher reality.",
            Style::default().fg(THEME.palette.log_info),
        ),
        ClientLog::Lost => ("I am lost...", Style::default().fg(THEME.palette.log_grave)),
        ClientLog::Win => (
            "I am whole again!",
            Style::default().fg(THEME.palette.log_info).bold(),
        ),
    }
}
