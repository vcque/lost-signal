use std::time::Duration;

use crossterm::terminal;
use lost_signal::common::{
    action::Action,
    sense::{SenseInfo, Senses, TerrainSense, WorldSense},
    types::{Direction, Offset, Tile},
};
use ratatui::{
    Terminal,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    prelude::Backend,
    style::{Color, Style},
    widgets::{Block, List, ListItem, ListState, Widget},
};

use crate::{game::GameSim, world::WorldView};

pub struct Tui {
    state: TuiState,
    page: Box<dyn Page>,
}

struct TuiState {
    game: GameSim,
    exit: bool,
}

impl Tui {
    pub fn new(game: GameSim) -> Self {
        Self {
            state: TuiState { game, exit: false },
            page: Box::new(MenuPage::default()),
        }
    }

    pub fn run(mut self) {
        let mut terminal = ratatui::init();
        terminal::enable_raw_mode().unwrap();

        std::panic::set_hook(Box::new(|_| {
            ratatui::restore();
            terminal::disable_raw_mode().unwrap();
        }));
        self.do_run(&mut terminal).unwrap();
        ratatui::restore();
        terminal::disable_raw_mode().unwrap();
    }

    fn do_run(
        &mut self,
        terminal: &mut Terminal<impl Backend>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while !self.state.exit {
            terminal.draw(|f| {
                self.page.render(f.area(), f.buffer_mut(), &self.state);
            })?;

            if event::poll(Duration::from_millis(100))? {
                let event = event::read()?;

                // Exit from anywhere
                match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        ..
                    }) => self.state.exit = true,
                    _ => {}
                }

                let nav = self.page.handle_events(event, &mut self.state);

                if let TuiNav::Goto(p) = nav {
                    self.page = p;
                }
            }
        }
        Ok(())
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
        state: &TuiState,
    ) {
        let menu_items: Vec<ListItem> = MENU_OPTIONS
            .iter()
            .map(|option| ListItem::new(option.to_string()))
            .collect();

        let menu_list = List::new(menu_items)
            .style(Style::default().fg(Color::Gray))
            .highlight_symbol("> ");

        ratatui::widgets::StatefulWidget::render(menu_list, area, buf, &mut self.list_state);
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
                            state.game.act(Action::Spawn, Senses::default());
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
        let [world_a, log_a, senses_a] = Self::layout(area);

        let world = state.game.world();

        let world_widget = WorldViewWidget {
            world: world.clone(),
        };
        let senses_widget = SensesWidget {
            selection: 0,
            senses: self.senses.clone(),
            info: world.last_info.clone(),
        };

        world_widget.render(world_a, buf);
        senses_widget.render(senses_a, buf);
        let logger_widget = tui_logger::TuiLoggerWidget::default().block(
            Block::default()
                .title("Logs")
                .borders(ratatui::widgets::Borders::ALL),
        );

        logger_widget.render(log_a, buf);
    }
    fn handle_events(&mut self, event: Event, tui: &mut TuiState) -> TuiNav {
        let Event::Key(key) = event else {
            return TuiNav::None;
        };

        match key.code {
            KeyCode::Up => {
                tui.game
                    .act(Action::Move(Direction::Up), self.senses.clone());
            }
            KeyCode::Down => {
                tui.game
                    .act(Action::Move(Direction::Down), self.senses.clone());
            }
            KeyCode::Left => {
                tui.game
                    .act(Action::Move(Direction::Left), self.senses.clone());
            }
            KeyCode::Right => {
                tui.game
                    .act(Action::Move(Direction::Right), self.senses.clone());
            }
            _ => {}
        }

        TuiNav::None
    }
}

/// Rendering of the map
struct WorldViewWidget {
    world: WorldView,
}

impl Widget for WorldViewWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let w = self.world;
        let center_x = area.width as isize / 2;
        let center_y = area.height as isize / 2;

        for x in 0..area.width {
            for y in 0..area.height {
                // Should take the tiles in batch ?
                let tile = w.tile_from_center(Offset {
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

/// Rendering of the used senses
#[derive(Debug)]
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
    }
}
