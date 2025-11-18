use std::sync::{Arc, Mutex};
use ratatui::Frame;

use crate::{
    game::GameSim,
    tui::{
        component::Component,
        pages::{GamePage, MenuPage},
        state::{ExternalState, GameState, MenuState, PageSelection, TuiState},
    },
    tui_adapter::{Event, TuiApp},
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

impl TuiApp for GameTui {
    fn render(&mut self, f: &mut Frame) {
        let area = f.area();
        let buf = f.buffer_mut();
        match self.state.page {
            PageSelection::Menu => MenuPage {}.render(area, buf, &mut self.state),
            PageSelection::Game => GamePage {}.render(area, buf, &mut self.state),
        };
    }

    fn handle_events(&mut self, event: Event) -> bool {
        match self.state.page {
            PageSelection::Menu => MenuPage {}.on_event(&event, &mut self.state),
            PageSelection::Game => GamePage {}.on_event(&event, &mut self.state),
        }
    }

    fn should_exit(&self) -> bool {
        self.state.should_exit
    }
}