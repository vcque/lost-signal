use std::sync::{Arc, Mutex};

use losig_core::{
    sense::{Senses, TerrainSense, WorldSense},
    types::{Action, Direction, Offset, Tile},
};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style},
    widgets::{List, ListItem, ListState, Widget},
};

use crate::{
    game::GameSim,
    tui_adapter::{Event, KeyCode, TuiApp},
    world::WorldView,
};

pub struct GameTui {
    state: TuiState,
    page: Box<dyn Page>,
}

struct TuiState {
    game: Arc<Mutex<GameSim>>,
    exit: bool,
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
}

impl Default for GamePage {
    fn default() -> Self {
        GamePage {
            senses: Senses {
                world: Some(WorldSense {}),
                terrain: Some(TerrainSense { radius: 5 }),
            },
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
        world_widget.render(world_a, buf);
    }
    fn handle_events(&mut self, event: Event, tui: &mut TuiState) -> TuiNav {
        let Event::Key(key) = event else {
            return TuiNav::None;
        };

        let mut game = tui.game.lock().unwrap();
        match key.code {
            KeyCode::Up => {
                game.act(Action::Move(Direction::Up), self.senses.clone());
            }
            KeyCode::Down => {
                game.act(Action::Move(Direction::Down), self.senses.clone());
            }
            KeyCode::Left => {
                game.act(Action::Move(Direction::Left), self.senses.clone());
            }
            KeyCode::Right => {
                game.act(Action::Move(Direction::Right), self.senses.clone());
            }
            _ => {}
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
