use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use losig_core::{
    sense::{SelfSense, SenseInfo, Senses, TerrainSense},
    types::{Action, Direction, Offset, Tile},
};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, List, ListItem, ListState, Widget},
};

use crate::{
    game::GameSim,
    sense::ClientSense,
    tui_adapter::{Event, KeyCode, TuiApp},
    world::WorldView,
};

pub struct GameTui {
    state: TuiState,
    page: Box<dyn Page>,
}

impl GameTui {
    pub fn new(game: Arc<Mutex<GameSim>>) -> Self {
        Self {
            state: TuiState { game, exit: false },
            page: Box::new(MenuPage::default()),
        }
    }
}

impl TuiApp for GameTui {
    fn render(&mut self, f: &mut Frame) {
        self.page.render(f.area(), f.buffer_mut(), &self.state);
    }

    fn handle_events(&mut self, event: crate::tui_adapter::Event) {
        let nav = self.page.handle_events(event, &mut self.state);

        if let TuiNav::Goto(p) = nav {
            self.page = p;
        }
    }

    fn should_exit(&self) -> bool {
        self.state.exit
    }
}

struct TuiState {
    game: Arc<Mutex<GameSim>>,
    exit: bool,
}

enum TuiNav {
    Goto(Box<dyn Page>),
    None,
}

impl TuiNav {
    fn from<T: Page + 'static>(page: T) -> TuiNav {
        TuiNav::Goto(Box::new(page))
    }
}

trait Page {
    fn handle_events(&mut self, event: Event, tui: &mut TuiState) -> TuiNav;

    fn render(
        &mut self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &TuiState,
    );
}

#[derive(Debug)]
struct MenuPage {
    list_state: ListState,
}

impl Default for MenuPage {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self { list_state }
    }
}

#[derive(Debug, Clone, Copy)]
enum MenuOption {
    Start,
    Continue,
}

impl Display for MenuOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            MenuOption::Start => "Start Game",
            MenuOption::Continue => "Continue Game",
        };
        f.write_str(string)
    }
}

const MENU_OPTIONS: &[MenuOption] = &[MenuOption::Start, MenuOption::Continue];

impl Page for MenuPage {
    fn render(
        &mut self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        _state: &TuiState,
    ) {
        let menu_items: Vec<ListItem> = MENU_OPTIONS
            .iter()
            .map(|option| ListItem::new(option.to_string()))
            .collect();

        let center = center(
            area,
            Constraint::Percentage(50),
            Constraint::Length(menu_items.len() as u16),
        );

        let menu_list = List::new(menu_items)
            .style(Style::default().fg(Color::Gray))
            .highlight_symbol("> ");

        ratatui::widgets::StatefulWidget::render(menu_list, center, buf, &mut self.list_state);
    }

    fn handle_events(&mut self, event: Event, state: &mut TuiState) -> TuiNav {
        let Event::Key(key) = event else {
            return TuiNav::None;
        };

        match key.code {
            KeyCode::Up => self.list_state.select_previous(),
            KeyCode::Down => self.list_state.select_next(),
            KeyCode::Enter => {
                if let Some(selection) = self.list_state.selected().map(|i| MENU_OPTIONS[i]) {
                    match selection {
                        MenuOption::Start => {
                            state
                                .game
                                .lock()
                                .unwrap()
                                .act(Action::Spawn, Senses::default());
                            return TuiNav::from(GamePage::default());
                        }
                        MenuOption::Continue => {
                            return TuiNav::from(GamePage::default());
                        }
                    }
                }
            }
            _ => {}
        }

        TuiNav::None
    }
}

#[derive(Debug)]
struct GamePage {
    senses: Senses,
    sense_selection: usize,
}

impl Default for GamePage {
    fn default() -> Self {
        GamePage {
            senses: Senses {
                selfs: Some(SelfSense {}),
                terrain: Some(TerrainSense { radius: 1 }),
                ..Default::default()
            },
            sense_selection: 0,
        }
    }
}

impl GamePage {
    fn layout(area: Rect) -> [Rect; 3] {
        let [main, senses] =
            Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
                .areas(area);

        let [world, log] =
            Layout::vertical([Constraint::Percentage(80), Constraint::Percentage(20)]).areas(main);

        [world, log, senses]
    }

    fn selected_sense_mut(&mut self) -> &mut dyn ClientSense {
        match self.sense_selection {
            0 => &mut self.senses.selfs,
            1 => &mut self.senses.terrain,
            2 => &mut self.senses.danger,
            _ => &mut self.senses.orb,
        }
    }
}

impl Page for GamePage {
    fn render(
        &mut self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &TuiState,
    ) {
        let [world_a, _log_a, _senses_a] = Self::layout(area);
        let game = state.game.lock().unwrap();
        let world = game.world();
        let world_widget = WorldViewWidget { world };

        let world_widget = Block::default()
            .borders(Borders::ALL)
            .title(format!(
                "World - turn {} - signal: {}/100",
                world.current_state().turn,
                world.current_state.signal
            ))
            .wrap(world_widget);

        world_widget.render(world_a, buf);

        let senses_widget = SensesWidget {
            senses: self.senses.clone(),
            info: world.last_info(),
            selection: self.sense_selection,
        };

        let cost = self.senses.signal_cost();
        let senses_wigdet = Block::default()
            .title(format!("Senses - cost: {}", cost))
            .wrap(senses_widget);
        senses_wigdet.render(_senses_a, buf);

        if world.winner {
            YouWinWidget {}.render(area, buf);
        }
    }

    fn handle_events(&mut self, event: Event, tui: &mut TuiState) -> TuiNav {
        let Event::Key(key) = event else {
            return TuiNav::None;
        };

        let mut game = tui.game.lock().unwrap();
        if game.world().winner {
            // No need to play once the game is won
            return TuiNav::None;
        }

        if key.modifiers.control {
            match key.code {
                KeyCode::Up => {
                    if self.sense_selection > 0 {
                        self.sense_selection -= 1;
                    }
                }
                KeyCode::Down => {
                    if self.sense_selection < 4 {
                        self.sense_selection += 1;
                    }
                }
                KeyCode::Right => {
                    self.selected_sense_mut().incr();
                }
                KeyCode::Left => {
                    self.selected_sense_mut().decr();
                }
                _ => {}
            }
        } else {
            let action = match key.code {
                KeyCode::Up | KeyCode::Char('8') => Some(Action::Move(Direction::Up)),
                KeyCode::Down | KeyCode::Char('2') => Some(Action::Move(Direction::Down)),
                KeyCode::Left | KeyCode::Char('4') => Some(Action::Move(Direction::Left)),
                KeyCode::Right | KeyCode::Char('6') => Some(Action::Move(Direction::Right)),
                KeyCode::Char('7') => Some(Action::Move(Direction::UpLeft)),
                KeyCode::Char('9') => Some(Action::Move(Direction::UpRight)),
                KeyCode::Char('1') => Some(Action::Move(Direction::DownLeft)),
                KeyCode::Char('3') => Some(Action::Move(Direction::DownRight)),
                KeyCode::Char('5') => Some(Action::Wait),
                KeyCode::Char('r') => Some(Action::Spawn),
                _ => None,
            };
            if let Some(action) = action {
                game.act(action, self.senses.clone());
            }
        }

        TuiNav::None
    }
}

/// Rendering of the map
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
                // Should take the tiles in batch ?
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
    status: &'a str,
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
        let first_line = format!(
            "{}{}{}",
            label,
            ".".repeat(area.width as usize - label.len() - indicator.len()),
            indicator
        );

        let second_line = format!("{:>width$}", status, width = area.width as usize);

        let selected_style = Style::default().green();
        let active_style = Style::default();
        let inactive_style = Style::default().gray();

        let first_line_style = match (selected, active) {
            (true, _) => selected_style,
            (_, true) => active_style,
            _ => inactive_style,
        };

        let second_line_style = if active { active_style } else { inactive_style };

        buf.set_string(area.x, area.y, &first_line, first_line_style);
        buf.set_string(area.x, area.y + 1, &second_line, second_line_style);
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
        let status = self
            .info
            .selfi
            .map(|w| if w.broken { "Broken" } else { "Intact" })
            .unwrap_or("-");
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
            .map(|t| format!("{} tiles", t.tiles.len()))
            .unwrap_or("-".to_owned());
        let indicator = match sense {
            Some(t) => format!("({})", t.radius),
            None => "(-)".to_owned(),
        };

        SenseWidget {
            label: "Terrain",
            indicator: indicator.as_str(),
            status: status.as_str(),
            selected: self.selection == row_index,
            active: sense.is_some(),
        }
        .render(rows[row_index], buf);
        row_index += 1;

        let sense = self.senses.danger;
        let status = self
            .info
            .danger
            .map(|prox| {
                if prox.count > 0 {
                    "Something's nearby"
                } else {
                    "Nothing"
                }
            })
            .unwrap_or("-");

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
        let status = self
            .info
            .orb
            .map(|orb| {
                if orb.detected {
                    "I can feel it"
                } else {
                    "Nothing"
                }
            })
            .unwrap_or("-");

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
            label: "Goal",
            indicator,
            status,
            selected: self.selection == row_index,
            active: sense.is_some(),
        }
        .render(rows[row_index], buf);
    }
}

/*
*
* Utils section
*
*/

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

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

trait BlockWrap<'a> {
    fn wrap<T: Widget>(self, widget: T) -> WBlock<'a, T>;
}

struct WBlock<'a, T> {
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

struct YouWinWidget {}

impl Widget for YouWinWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let message = "YOU WIN";
        let popup_width = 30;
        let popup_height = 7;

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
            .title("Game Over")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::White));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        // Center the message in the popup
        let text_y = inner.y + inner.height / 2;
        let text_x = inner.x + (inner.width.saturating_sub(message.len() as u16)) / 2;

        let text_style = Style::default().fg(Color::Green);
        buf.set_string(text_x, text_y, message, text_style);
    }
}
