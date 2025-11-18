use std::sync::{Arc, Mutex};
use losig_core::sense::{SelfSense, Senses, TerrainSense};
use ratatui::widgets::ListState;

use crate::{game::GameSim, sense::ClientSense};

pub struct TuiState {
    pub external: ExternalState,
    pub menu: MenuState,
    pub game: GameState,
    pub page: PageSelection,
    pub should_exit: bool,
}

pub struct ExternalState {
    pub game: Arc<Mutex<GameSim>>,
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
        }
    }
}