use losig_core::sense::{SenseStrength, Senses};
use ratatui::widgets::ListState;

use crate::tui::widgets::help::HelpState;

pub struct TuiState {
    pub menu: MenuState,
    pub game: GameState,
    pub you_win: GameOverState,
    pub limbo: LimboState,
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
    pub entering_name: bool,
    pub name: String,
}

impl Default for MenuState {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            list_state,
            entering_name: false,
            name: String::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct GameState {
    pub senses: Senses,
    pub sense_selection: usize,
    pub help: HelpState,
}

impl GameState {
    pub fn decr_sense(&mut self, available_senses: &[losig_core::sense::SenseType]) {
        use losig_core::sense::SenseType;

        let Some(&sense_type) = available_senses.get(self.sense_selection) else {
            return;
        };

        let senses = &mut self.senses;
        match sense_type {
            SenseType::SelfSense => senses.selfs = senses.selfs.decr(),
            SenseType::Touch => senses.touch = senses.touch.decr(),
            SenseType::Hearing => senses.hearing = senses.hearing.decr(),
            SenseType::Sight => senses.sight = senses.sight.decr(),
        }
    }

    pub fn incr_sense(&mut self, available_senses: &[losig_core::sense::SenseType]) {
        use losig_core::sense::SenseType;

        let Some(&sense_type) = available_senses.get(self.sense_selection) else {
            return;
        };

        let senses = &mut self.senses;
        match sense_type {
            SenseType::SelfSense => senses.selfs = senses.selfs.incr(),
            SenseType::Touch => senses.touch = senses.touch.incr(),
            SenseType::Hearing => senses.hearing = senses.hearing.incr(),
            SenseType::Sight => senses.sight = senses.sight.incr(),
        }
    }
}

#[derive(Debug, Default)]
pub struct GameOverState {
    pub open: bool,
    pub name: String,
    pub sent: bool,
}

#[derive(Debug, Default)]
pub struct LimboState {
    pub open: bool,
    pub averted: bool,
}
