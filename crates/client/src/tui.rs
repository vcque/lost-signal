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
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Widget},
};

use crate::{
    game::GameSim,
    sense::ClientSense,
    theme::THEME,
    tui_adapter::{Event, KeyCode, TuiApp},
    world::WorldView,
};

pub struct GameTui {
    state: TuiState,
}

impl GameTui {
    pub fn new(game: Arc<Mutex<GameSim>>) -> Self {
        Self {
            state: TuiState {
                external: ExternalState { game },
                menu: MenuState::default(),
                game: GameState::default(),
                page: PageSelection::Menu,
                should_exit: false,
            },
        }
    }
}

trait Component {
    type State;
    fn on_event(self, event: &Event, state: &mut Self::State) -> bool;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State);
}

impl TuiApp for GameTui {
    fn render(&mut self, f: &mut Frame) {
        let area = f.area();
        let buf = f.buffer_mut();
        match self.state.page {
            PageSelection::Menu => MenuPage {}.render(area, buf, &mut self.state),
            PageSelection::Game => GamePage {}.render(area, buf, &mut self.state),
        };
    }

    fn handle_events(&mut self, event: crate::tui_adapter::Event) -> bool {
        match self.state.page {
            PageSelection::Menu => MenuPage {}.on_event(&event, &mut self.state),
            PageSelection::Game => GamePage {}.on_event(&event, &mut self.state),
        }
    }

    fn should_exit(&self) -> bool {
        self.state.should_exit
    }
}

struct TuiState {
    external: ExternalState,
    menu: MenuState,
    game: GameState,
    page: PageSelection,
    should_exit: bool,
}

struct ExternalState {
    game: Arc<Mutex<GameSim>>,
}

enum PageSelection {
    Menu,
    Game,
}

#[derive(Debug)]
struct MenuState {
    list_state: ListState,
}

impl Default for MenuState {
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

struct MenuPage {}

impl Component for MenuPage {
    type State = TuiState;

    fn on_event(self, event: &Event, state: &mut Self::State) -> bool {
        let Event::Key(key) = event else {
            return false;
        };

        let list_state = &mut state.menu.list_state;
        match key.code {
            KeyCode::Up => list_state.select_previous(),
            KeyCode::Down => list_state.select_next(),
            KeyCode::Enter => {
                if let Some(selection) = list_state.selected().map(|i| MENU_OPTIONS[i]) {
                    match selection {
                        MenuOption::Start => {
                            state
                                .external
                                .game
                                .lock()
                                .unwrap()
                                .act(Action::Spawn, Senses::default());
                        }
                        MenuOption::Continue => {}
                    }
                    state.page = PageSelection::Game;
                }
            }
            _ => {
                return false;
            }
        }

        return true;
    }

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
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

        ratatui::widgets::StatefulWidget::render(
            menu_list,
            center,
            buf,
            &mut state.menu.list_state,
        );
    }
}

impl MenuPage {}

#[derive(Debug)]
struct GameState {
    senses: Senses,
    sense_selection: usize,
}

impl GameState {
    fn selected_sense_mut(&mut self) -> &mut dyn ClientSense {
        match self.sense_selection {
            0 => &mut self.senses.selfs,
            1 => &mut self.senses.terrain,
            2 => &mut self.senses.danger,
            _ => &mut self.senses.orb,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            senses: Senses {
                selfs: Some(SelfSense {}),
                terrain: Some(TerrainSense { radius: 1 }),
                ..Default::default()
            },
            sense_selection: 0,
        }
    }
}

struct GamePage {}

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
        let [world_a, _log_a, _senses_a] = Self::layout(area);
        let game = state.external.game.lock().unwrap();
        let world = game.world();
        let world_widget = WorldViewWidget { world };

        let world_title = Line::from(vec![
            Span::raw(format!(
                "World - turn {} - signal: ",
                world.current_state().turn
            )),
            Span::styled(
                format!("{}/100", world.current_state.signal),
                THEME.styles.signal,
            ),
        ]);

        Block::default()
            .borders(Borders::ALL)
            .title(world_title)
            .wrap(world_widget)
            .render(world_a, buf);

        let state = &mut state.game;
        let senses_widget = SensesWidget {
            senses: state.senses.clone(),
            info: world.last_info(),
            selection: state.sense_selection,
        };

        let cost = state.senses.signal_cost();
        let cost_style = if cost > world.current_state.signal {
            Style::default().red().bold()
        } else {
            Style::default()
        };

        let title = Line::from(vec![
            Span::raw("Senses - cost: "),
            Span::styled(cost.to_string(), cost_style),
        ]);

        let senses_wigdet = Block::default().title(title).wrap(senses_widget);
        senses_wigdet.render(_senses_a, buf);

        if world.winner {
            YouWinWidget {}.render(area, buf);
        }
    }

    fn on_event(self, event: &Event, state: &mut Self::State) -> bool {
        let Event::Key(key) = event else {
            return false;
        };

        let mut game = state.external.game.lock().unwrap();
        if game.world().winner {
            // No need to play once the game is won
            return false;
        }

        let state = &mut state.game;
        if key.modifiers.control {
            match key.code {
                KeyCode::Up => {
                    if state.sense_selection > 0 {
                        state.sense_selection -= 1;
                    }
                }
                KeyCode::Down => {
                    if state.sense_selection < 4 {
                        state.sense_selection += 1;
                    }
                }
                KeyCode::Right => {
                    state.selected_sense_mut().incr();
                }
                KeyCode::Left => {
                    state.selected_sense_mut().decr();
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
                game.act(action, state.senses.clone());
                return true;
            }
        }
        false
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
            1 => Line::from("There's something nearby").style(THEME.styles.danger),
            n => Line::from(format!("There's {} dangers nearby", n)).style(THEME.styles.danger),
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
                Line::from("Nothing.")
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
