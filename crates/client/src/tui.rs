use std::sync::{Arc, Mutex};

use losig_core::{
    sense::{SenseInfo, Senses, TerrainSense, WorldSense},
    types::{Action, Direction, EntityId, Offset, Tile},
};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, List, ListItem, ListState, Widget},
};

use crate::{
    game::GameSim,
    sense::Sense,
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
        return self.state.exit;
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
    Exit,
}

impl ToString for MenuOption {
    fn to_string(&self) -> String {
        match self {
            MenuOption::Start => "Start Game",
            MenuOption::Continue => "Continue Game",
            MenuOption::Exit => "Exit",
        }
        .to_owned()
    }
}

const MENU_OPTIONS: [MenuOption; 3] = [MenuOption::Start, MenuOption::Continue, MenuOption::Exit];

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
                        MenuOption::Exit => {
                            // Exit
                            state.exit = true;
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
                world: Some(WorldSense {}),
                terrain: Some(TerrainSense { radius: 5 }),
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

    fn selected_sense_mut(&mut self) -> &mut dyn Sense {
        match self.sense_selection {
            0 => &mut self.senses.world as &mut dyn Sense,
            _ => &mut self.senses.terrain as &mut dyn Sense,
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
        let world_widget = WorldViewWidget { world: &world };

        let world_widget = Block::default()
            .borders(Borders::ALL)
            .title("World")
            .wrap(world_widget);

        world_widget.render(world_a, buf);

        let senses_widget = SensesWidget {
            senses: self.senses,
            info: world.last_info.clone(),
            selection: self.sense_selection,
        };

        let senses_wigdet = Block::default().title("Senses").wrap(senses_widget);
        senses_wigdet.render(_senses_a, buf);

        if let Some(entity_id) = world.last_info.world.as_ref().and_then(|w| w.winner) {
            YouWinWidget {
                winner: entity_id,
                me: game.entity_id,
            }
            .render(area, buf);
        }
    }

    fn handle_events(&mut self, event: Event, tui: &mut TuiState) -> TuiNav {
        let Event::Key(key) = event else {
            return TuiNav::None;
        };

        let mut game = tui.game.lock().unwrap();
        if game
            .world()
            .last_info
            .world
            .as_ref()
            .and_then(|w| w.winner)
            .is_some()
        {
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
                    if self.sense_selection < 2 {
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
                let tile = w.tile_from_viewer(Offset {
                    x: x as isize - center_x,
                    y: y as isize - center_y,
                });

                buf.set_string(
                    area.x + x,
                    area.y + y,
                    to_char(tile).to_string(),
                    Style::default(),
                );
            }
        }

        buf.set_string(
            area.x + center_x as u16,
            area.y + center_y as u16,
            "@",
            Style::default(),
        );
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
        let label = "World";
        let active = self.senses.world.is_some();
        let status = self
            .info
            .world
            .map(|w| format!("turn {}", w.tick))
            .unwrap_or("()".to_owned());

        let indicator = if active { "(+)" } else { "(-)" };
        let first_line = format!(
            "{}{}{}",
            label,
            ".".repeat(area.width as usize - label.len() - indicator.len()),
            indicator
        );

        let second_line = format!("{:>width$}", status, width = area.width as usize);

        let style = if self.selection == 0 {
            Style::default().bold().fg(Color::LightGreen)
        } else {
            Style::default()
        };
        buf.set_string(area.x, area.y, &first_line, style);
        buf.set_string(area.x, area.y + 1, &second_line, Style::default());

        let label = "Terrain";
        let active = self.senses.terrain.is_some();
        let status = self
            .info
            .terrain
            .map(|t| format!("{} tiles", t.tiles.len()))
            .unwrap_or("-".to_owned());

        let indicator = match self.senses.terrain {
            Some(t) => format!("({})", t.radius),
            None => "(-)".to_owned(),
        };

        let first_line = format!(
            "{}{}{}",
            label,
            ".".repeat(area.width as usize - label.len() - indicator.len()),
            indicator
        );

        let second_line = format!("{:>width$}", status, width = area.width as usize);
        let style = if self.selection == 1 {
            Style::default().bold().fg(Color::LightGreen)
        } else {
            Style::default()
        };
        buf.set_string(area.x, area.y + 2, &first_line, style);
        buf.set_string(area.x, area.y + 3, &second_line, Style::default());
    }
}

/*
*
* Utils section
*
*/

fn to_char(tile: Tile) -> char {
    match tile {
        Tile::Spawn => 'S',
        Tile::Wall => '#',
        Tile::Unknown => ' ',
        Tile::Empty => '.',
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

struct YouWinWidget {
    me: EntityId,
    winner: EntityId,
}

impl Widget for YouWinWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let message = if self.me == self.winner {
            "YOU WIN!"
        } else {
            "YOU LOSE."
        };

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

        let text_style = if self.me == self.winner {
            Style::default().fg(Color::Green).bold()
        } else {
            Style::default().fg(Color::Red).bold()
        };

        buf.set_string(text_x, text_y, message, text_style);
    }
}
