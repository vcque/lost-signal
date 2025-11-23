use losig_core::sense::{SenseStrength, Senses};
use ratatui::widgets::ListState;

pub struct TuiState {
    pub menu: MenuState,
    pub game: GameState,
    pub you_win: YouWinState,
    pub page: PageSelection,
    pub should_exit: bool,
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

#[derive(Debug, Default)]
pub struct GameState {
    pub senses: Senses,
    pub stage: u8,
    pub sense_selection: usize,
    pub show_help: bool,
}

impl GameState {
    pub fn decr_sense(&mut self) {
        let senses = &mut self.senses;
        match self.sense_selection {
            0 => senses.selfs = senses.selfs.decr(),
            1 => senses.touch = senses.touch.decr(),
            2 => senses.hearing = senses.hearing.decr(),
            3 => senses.sight = senses.sight.decr(),
            _ => {}
        }
    }

    pub fn incr_sense(&mut self) {
        let senses = &mut self.senses;
        match self.sense_selection {
            0 => senses.selfs = senses.selfs.incr(),
            1 => senses.touch = senses.touch.incr(),
            2 => senses.hearing = senses.hearing.incr(),
            3 => senses.sight = senses.sight.incr(),
            _ => {}
        }
    }
}

#[derive(Debug, Default)]
pub struct YouWinState {
    pub open: bool,
    pub name: String,
    pub sent: bool,
}
