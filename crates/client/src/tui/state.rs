use losig_core::{
    leaderboard::Leaderboard,
    network::{ClientMessage, ClientMessageContent, CommandMessage},
    sense::{SenseStrength, Senses},
    types::Action,
};
use ratatui::widgets::ListState;
use std::sync::{Arc, Mutex};

use crate::{adapter::Client, world::WorldView};

pub struct TuiState {
    pub external: ExternalState,
    pub menu: MenuState,
    pub game: GameState,
    pub you_win: YouWinState,
    pub page: PageSelection,
    pub should_exit: bool,
}

pub struct ExternalState {
    pub client: Arc<Mutex<dyn Client>>,
    pub world: Arc<Mutex<WorldView>>,
    pub leaderboard: Arc<Mutex<Leaderboard>>,
}

impl ExternalState {
    pub fn act(&self, action: Action, senses: Senses) {
        let mut world = self.world.lock().unwrap();
        world.act(&action);
        let client = self.client.lock().unwrap();
        client.send(ClientMessage {
            avatar_id: Some(world.avatar_id),
            content: losig_core::network::ClientMessageContent::Command(CommandMessage {
                avatar_id: world.avatar_id,
                turn: world.turn,
                action,
                senses,
            }),
        });
    }

    pub fn submit_leaderboard(&self, name: String) {
        let world = self.world.lock().unwrap();
        let client = self.client.lock().unwrap();
        client.send(ClientMessage {
            avatar_id: Some(world.avatar_id),
            content: ClientMessageContent::LeaderboardSubmit(world.avatar_id, name),
        });
    }
}

#[derive(Debug)]
pub enum PageSelection {
    Menu,
    Game,
}

#[derive(Debug)]
pub struct MenuState {
    pub list_state: ListState,
}

impl Default for MenuState {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self { list_state }
    }
}

#[derive(Debug)]
pub struct GameState {
    pub senses: Senses,
    pub sense_selection: usize,
    pub show_help: bool,
}

impl GameState {
    pub fn decr_sense(&mut self) {
        let senses = &mut self.senses;
        match self.sense_selection {
            0 => senses.selfs = senses.selfs.decr(),
            1 => senses.touch = senses.touch.decr(),
            2 => senses.sight = senses.sight.decr(),
            _ => {}
        }
    }

    pub fn incr_sense(&mut self) {
        let senses = &mut self.senses;
        match self.sense_selection {
            0 => senses.selfs = senses.selfs.incr(),
            1 => senses.touch = senses.touch.incr(),
            2 => senses.sight = senses.sight.incr(),
            _ => {}
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            senses: Senses {
                selfs: true,
                touch: true,
                ..Default::default()
            },
            sense_selection: 0,
            show_help: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct YouWinState {
    pub open: bool,
    pub name: String,
    pub sent: bool,
}
