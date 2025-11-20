use losig_core::leaderboard::Leaderboard;
use ratatui::Frame;
use std::sync::{Arc, Mutex};

use crate::{
    adapter::Client,
    tui::{
        component::Component,
        pages::{GamePage, MenuPage},
        state::{ExternalState, GameState, MenuState, PageSelection, TuiState, YouWinState},
    },
    tui_adapter::{Event, TuiApp},
    world::WorldView,
};

pub struct GameTui {
    state: TuiState,
}

impl GameTui {
    pub fn new(
        client: Arc<dyn Client>,
        world: Arc<Mutex<WorldView>>,
        leaderboard: Arc<Mutex<Leaderboard>>,
    ) -> Self {
        Self {
            state: TuiState {
                external: ExternalState {
                    client,
                    world,
                    leaderboard,
                },
                menu: MenuState::default(),
                game: GameState::default(),
                you_win: YouWinState::default(),
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

