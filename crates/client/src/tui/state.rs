use losig_core::{
    leaderboard::Leaderboard,
    network::{ClientMessage, CommandMessage},
    sense::{SelfSense, Senses, TerrainSense},
    types::Action,
};
use ratatui::widgets::ListState;
use std::sync::{Arc, Mutex};

use crate::{adapter::Client, sense::ClientSense, world::WorldView};

pub struct TuiState {
    pub external: ExternalState,
    pub menu: MenuState,
    pub game: GameState,
    pub page: PageSelection,
    pub should_exit: bool,
}

pub struct ExternalState {
    pub client: Arc<dyn Client>,
    pub world: Arc<Mutex<WorldView>>,
    pub leaderboard: Arc<Mutex<Leaderboard>>,
}

impl ExternalState {
    pub fn act(&self, action: Action, senses: Senses) {
        let mut world = self.world.lock().unwrap();
        world.act(&action, &senses);
        self.client.send(ClientMessage {
            avatar_id: Some(world.avatar_id),
            content: losig_core::network::ClientMessageContent::Command(CommandMessage {
                avatar_id: world.avatar_id,
                turn: world.turn,
                action,
                senses,
            }),
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
    pub fn selected_sense_mut(&mut self) -> &mut dyn ClientSense {
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
            show_help: false,
        }
    }
}

