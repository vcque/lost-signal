use itertools::Itertools;
use log::info;
use losig_core::{
    sense::SightedAllyStatus,
    types::{ClientAction, Direction, FoeType, GameOver, GameOverStatus, Offset, Tile},
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Clear, Widget},
};

use crate::{
    tui::{
        GameOverState, InputServices, RenderServices, THEME, ally_color,
        state::{LimboState, TuiState},
        utils::center,
        widgets::{
            block_wrap::BlockWrap, help::HelpWidget, logs::LogsWidget, senses::SensesWidget,
            timeline::TimelineWidget,
        },
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
        state.game.help.next_page(world.stage_id as u8);

        let world_widget = WorldViewWidget { world };
        let timeline = TimelineWidget::new(world);

        Block::default()
            .borders(Borders::ALL)
            .title(timeline)
            .wrap(world_widget)
            .render(world_a, buf);

        let logs_widget = LogsWidget {
            logs: world.logs.logs(),
            current_turn: world.turn,
        };
        Block::default()
            .borders(Borders::ALL)
            .title("Game Log")
            .wrap(logs_widget)
            .render(log_a, buf);

        let game_state = &mut state.game;
        let senses_widget = SensesWidget {
            stage_turn: world.stage_turn,
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

        let (cost_style, title) = if world.last_info().is_none() {
            let tired_style = Style::default().fg(Color::White).bg(Color::Red);
            let tired_title = Line::from(" TIRED ").alignment(ratatui::layout::Alignment::Center);
            (tired_style, tired_title)
        } else {
            let style = if focus.is_some_and(|s| s < cost) {
                Style::default().bold().bg(THEME.palette.foe)
            } else {
                Style::default()
            };

            let focus_str = focus.map(|s| s.to_string()).unwrap_or("??".to_owned());
            let cost_title = Line::from(format!(" Senses - cost: {} / {} ", cost, focus_str))
                .alignment(ratatui::layout::Alignment::Center);
            (style, cost_title)
        };

        let senses_wigdet = Block::default()
            .borders(Borders::TOP)
            .title(title)
            .border_style(cost_style)
            .wrap(senses_widget);
        senses_wigdet.render(_senses_a, buf);

        // Display latency at the bottom of the senses panel
        if let Some(latency) = world.last_latency {
            let latency_text = format!("Latency: {}ms", latency.as_millis());
            let latency_style = Style::default().fg(Color::Black).bg(Color::White);
            let bottom_y = _senses_a.y + _senses_a.height.saturating_sub(1);
            let latency_area = Rect::new(_senses_a.x, bottom_y, _senses_a.width, 1);

            Line::from(latency_text)
                .style(latency_style)
                .render(latency_area, buf);
        }

        // Check for limbo state from server
        if let Some(averted) = services.state.limbo {
            state.limbo.open = true;
            state.limbo.averted = averted;
        }

        if let Some(gameover) = &services.state.gameover {
            GameOverWidget {}.render(area, buf, gameover, &mut state.you_win);
        } else if state.limbo.open {
            LimboWidget {}.render(area, buf, state.limbo.averted, &mut state.limbo);
        } else if state.game.help.open {
            HelpWidget.render(area, buf, &state.game.help);
        }
    }

    pub fn on_event(
        self,
        event: &Event,
        state: &mut TuiState,
        mut services: InputServices,
    ) -> bool {
        if let Some(gameover) = services.state.gameover.clone()
            && (GameOverWidget {}).on_event(event, &mut state.you_win, &mut services, &gameover)
        {
            return true;
        }
        // If limbo is visible, let LimboWidget handle the event
        if state.limbo.open {
            return LimboWidget {}.on_event(event, &mut state.limbo, &mut services);
        }

        // If help is visible, let HelpWidget handle the event
        if state.game.help.open {
            return HelpWidget.on_event(event, &mut state.game.help);
        }

        let Event::Key(key) = event else {
            return false;
        };

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
                Some(ClientAction::MoveOrAttack(Direction::Up))
            }
            KeyCode::Down | KeyCode::Char('2') | KeyCode::Char('j') => {
                Some(ClientAction::MoveOrAttack(Direction::Down))
            }
            KeyCode::Left | KeyCode::Char('4') | KeyCode::Char('h') => {
                Some(ClientAction::MoveOrAttack(Direction::Left))
            }
            KeyCode::Right | KeyCode::Char('6') | KeyCode::Char('l') => {
                Some(ClientAction::MoveOrAttack(Direction::Right))
            }
            KeyCode::Char('7') | KeyCode::Char('y') => {
                Some(ClientAction::MoveOrAttack(Direction::UpLeft))
            }
            KeyCode::Char('9') | KeyCode::Char('u') => {
                Some(ClientAction::MoveOrAttack(Direction::UpRight))
            }
            KeyCode::Char('1') | KeyCode::Char('b') => {
                Some(ClientAction::MoveOrAttack(Direction::DownLeft))
            }
            KeyCode::Char('3') | KeyCode::Char('n') => {
                Some(ClientAction::MoveOrAttack(Direction::DownRight))
            }
            KeyCode::Char('5') | KeyCode::Char(' ') => Some(ClientAction::Wait),
            KeyCode::Char('?') => {
                game_state.help.open = true;
                return true;
            }
            _ => None,
        };
        if let Some(action) = action {
            // Check for wall collision before moving
            if let ClientAction::MoveOrAttack(dir) = &action {
                let new_pos = services.state.world.current_state.position + dir.offset();
                let tile = services.state.world.current_state.tile_at(new_pos);
                if !tile.can_travel() {
                    // Cancel move into wall
                    return true;
                }
            }

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
        Tile::Spawn => ('_', Style::new().fg(THEME.palette.important)),
        Tile::Wall => ('█', Style::new().fg(THEME.palette.tile_wall)),
        Tile::Unknown => (' ', *DEFAULT_STYLE),
        Tile::Empty => ('.', Style::new().fg(THEME.palette.tile_floor)),
        Tile::Pylon => ('|', Style::new().fg(THEME.palette.important)),
        Tile::StairUp => ('<', Style::new().fg(THEME.palette.tile_stair)),
        Tile::StairDown => ('>', Style::new().fg(THEME.palette.tile_stair)),
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
                    style.fg(THEME.palette.tile_unseen)
                } else if style.bg.is_some() {
                    style.bg(THEME.palette.tile_unseen)
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
            for foe in sight.foes.iter().sorted_by_key(|f| f.alive) {
                let x = center_x + foe.offset.x;
                let y = center_y + foe.offset.y;

                let char = match foe.foe_type {
                    FoeType::Simple => "s",
                    FoeType::MindSnare => "¤",
                };

                let style = if foe.alive {
                    Style::default().fg(THEME.palette.foe)
                } else {
                    Style::default().fg(THEME.palette.ui_disabled)
                };

                buf.set_string(area.x + x as u16, area.y + y as u16, char, style);
            }

            // Show the allies
            for ally in &sight.allies {
                let x = center_x + ally.offset.x;
                let y = center_y + ally.offset.y;

                let color = match ally.status {
                    SightedAllyStatus::Controlled { turn, .. } => ally_color(turn, w.stage_turn),
                    SightedAllyStatus::Discarded => THEME.palette.ally_discarded,
                };
                buf.set_string(area.x + x as u16, area.y + y as u16, "@", color);

                if let Some(offset) = ally.next_move {
                    info!(
                        "{}; {}; {}| {}, {}, {}",
                        area.x, center_x, offset.x, area.y, center_y, offset.y
                    );
                    let x = area.x + (center_x + offset.x) as u16;
                    let y = area.y + (center_y + offset.y) as u16;
                    let area = Rect::new(x, y, 1, 1);
                    buf.set_style(area, Style::default().bg(THEME.palette.ally_next_move));
                }
            }
        }

        let touch_info = last_info.and_then(|i| i.touch.as_ref());
        let has_sight = last_info.and_then(|i| i.sight.as_ref()).is_some();

        // Render touched foes as "?" when sight is inactive
        if !has_sight && let Some(touch) = touch_info {
            for offset in &touch.foes {
                let x = center_x + offset.x;
                let y = center_y + offset.y;
                buf.set_string(area.x + x as u16, area.y + y as u16, "?", THEME.palette.foe);
            }
        }

        let neigboring_traps = touch_info.map(|it| it.traps).unwrap_or_default();
        if !has_sight && neigboring_traps > 0 {
            buf.set_string(
                area.x + center_x as u16,
                area.y + center_y as u16,
                neigboring_traps.to_string(),
                THEME.palette.trap,
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

struct GameOverWidget {}

impl GameOverWidget {
    pub fn render(
        self,
        area: Rect,
        buf: &mut Buffer,
        gameover: &GameOver,
        state: &mut GameOverState,
    ) {
        let popup_width = 50;
        let popup_height = 10;

        let popup_area = center(
            area,
            Constraint::Length(popup_width),
            Constraint::Length(popup_height),
        );

        // Clear the popup area to reset style
        Clear.render(popup_area, buf);

        let (title, color) = match gameover.status {
            GameOverStatus::Win => ("🎉 Victory! 🎉", THEME.palette.log_info),
            GameOverStatus::Dead => ("💀 Game Over 💀", THEME.palette.log_grave),
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
        state: &mut GameOverState,
        services: &mut InputServices,
        _gameover: &GameOver,
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

struct LimboWidget {}

impl LimboWidget {
    pub fn render(self, area: Rect, buf: &mut Buffer, averted: bool, _state: &mut LimboState) {
        let popup_width = 50;
        let popup_height = 10;

        let popup_area = center(
            area,
            Constraint::Length(popup_width),
            Constraint::Length(popup_height),
        );

        // Clear the popup area to reset style
        Clear.render(popup_area, buf);

        let (title, color, lines) = if averted {
            (
                "✨ Saved ✨",
                THEME.palette.log_info,
                vec![
                    "Another player saved you!",
                    "",
                    "(Press any key to continue)",
                ],
            )
        } else {
            (
                "Incapacited",
                Color::Cyan,
                vec![
                    "You have died...",
                    "",
                    "You are now in limbo, waiting for",
                    "another player to save you.",
                    "",
                    "If no one saves you, the game is over.",
                ],
            )
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default().fg(color));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        for (i, line) in lines.iter().enumerate() {
            let y = inner.y + 1 + i as u16;
            let x = inner.x + (inner.width.saturating_sub(line.len() as u16)) / 2;
            let style = if i == 0 {
                Style::default().fg(THEME.palette.important)
            } else {
                Style::default().fg(THEME.palette.ui_text)
            };
            buf.set_string(x, y, line, style);
        }
    }

    fn on_event(
        self,
        _event: &Event,
        state: &mut LimboState,
        services: &mut InputServices,
    ) -> bool {
        if state.averted {
            // Any key dismisses the popup
            state.open = false;
            services.clear_limbo();
            true
        } else {
            false
        }
    }
}
