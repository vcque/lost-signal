use losig_core::{
    network::{ClientMessage, ClientMessageContent, CommandMessage},
    sense::Senses,
    types::Action,
};
use ratatui::Frame;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::{
    adapter::{Client, SharedState},
    tui::{
        pages::{GamePage, MenuPage},
        state::{GameOverState, GameState, LimboState, MenuState, PageSelection, TuiState},
    },
    tui_adapter::Event,
};

pub struct GameTui {
    state: TuiState,
    external: ExternalServices,
}

struct ExternalServices {
    state: Arc<Mutex<SharedState>>,
    client: Arc<Mutex<dyn Client>>,
}

impl ExternalServices {
    fn render_services<'a>(&'a self) -> RenderServices<'a> {
        RenderServices {
            state: self.state.lock().unwrap(),
        }
    }

    fn input_services<'a>(&'a self) -> InputServices<'a> {
        InputServices {
            state: self.state.lock().unwrap(),
            client: self.client.lock().unwrap(),
        }
    }
}

/// Services accessible when the game is rendered
pub struct RenderServices<'a> {
    pub state: MutexGuard<'a, SharedState>,
}

/// Services accessible when the user inputs something
pub struct InputServices<'a> {
    pub state: MutexGuard<'a, SharedState>,
    pub client: MutexGuard<'a, dyn Client>,
}

impl<'a> InputServices<'a> {
    pub fn act(&mut self, action: Action, senses: Senses) {
        self.state.world.act(&action);
        let avatar_id = self.state.avatar_id;
        self.client.send(ClientMessage {
            avatar_id: Some(avatar_id),
            content: ClientMessageContent::Command(CommandMessage {
                avatar_id,
                turn: self.state.world.turn,
                action,
                senses,
            }),
        });
    }

    pub fn submit_leaderboard(&self, name: String) {
        self.client.send(ClientMessage {
            avatar_id: Some(self.state.avatar_id),
            content: ClientMessageContent::LeaderboardSubmit(self.state.avatar_id, name),
        });
    }

    pub fn new_game(&self) {
        self.client.send(ClientMessage {
            avatar_id: Some(self.state.avatar_id),
            content: ClientMessageContent::Start(self.state.avatar_id),
        });
    }

    pub fn clear_gameover(&mut self) {
        self.state.gameover = None;
    }

    pub fn clear_limbo(&mut self) {
        self.state.limbo = None;
    }
}

impl GameTui {
    pub fn new(client: Arc<Mutex<dyn Client>>, shared_state: Arc<Mutex<SharedState>>) -> Self {
        Self {
            external: ExternalServices {
                state: shared_state,
                client,
            },
            state: TuiState {
                menu: MenuState::default(),
                game: GameState::default(),
                you_win: GameOverState::default(),
                limbo: LimboState::default(),
                page: PageSelection::Menu,
                should_exit: false,
            },
        }
    }
}

impl GameTui {
    pub fn render(&mut self, f: &mut Frame) {
        let area = f.area();
        let buf = f.buffer_mut();

        let services = self.external.render_services();

        match self.state.page {
            PageSelection::Menu => MenuPage {}.render(area, buf, &mut self.state.menu, services),
            PageSelection::Game => GamePage {}.render(area, buf, &mut self.state, services),
        };
    }

    pub fn handle_events(&mut self, event: Event) -> bool {
        let services = self.external.input_services();
        match self.state.page {
            PageSelection::Menu => MenuPage {}.on_event(&event, &mut self.state, services),
            PageSelection::Game => GamePage {}.on_event(&event, &mut self.state, services),
        }
    }

    pub fn should_exit(&self) -> bool {
        self.state.should_exit
    }
}
