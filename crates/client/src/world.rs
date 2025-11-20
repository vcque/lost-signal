use log::warn;
use losig_core::{
    sense::{SenseInfo, Senses},
    types::{Action, AvatarId, Offset, Position, Tile},
};

use crate::logs::{ClientLog, GameLogs};

const VIEW_SIZE: usize = 256;
const START_POS: Position = Position {
    x: VIEW_SIZE / 2,
    y: VIEW_SIZE / 2,
};

#[derive(Debug, Clone)]
pub struct WorldView {
    pub avatar_id: AvatarId,
    pub winner: bool,
    pub stage: usize,
    pub turn: u64,

    history: Vec<WorldHistory>,
    past_state: WorldState,
    pub current_state: WorldState,
    pub logs: GameLogs,
}

impl WorldView {
    pub fn new(id: AvatarId) -> Self {
        let mut logs = GameLogs::default();
        logs.add(1, ClientLog::Help);

        Self {
            avatar_id: id,
            winner: false,
            stage: 0,
            turn: 1,
            history: vec![],
            past_state: WorldState::default(),
            current_state: WorldState::default(),
            logs,
        }
    }

    pub fn act(&mut self, action: &Action, senses: &Senses) {
        if matches!(action, Action::Spawn) {
            self.clear();
        }

        let history = WorldHistory {
            action: *action,
            senses: senses.clone(),
            info: None,
        };
        self.current_state.update(&history);
        self.history.push(history);

        // Don't maintain old history entries
        let to_remove = self.history.len().saturating_sub(10);
        for history in self.history.drain(0..to_remove) {
            self.past_state.update(&history);
        }

        self.turn += 1;
    }

    pub fn update(&mut self, turn: u64, info: SenseInfo) {
        let diff = turn.abs_diff(self.turn);
        // Update global info
        if diff == 0
            && let Some(ref selfi) = info.selfi
        {
            if self.winner != selfi.winner {
                self.logs.add(turn, ClientLog::Win);
            }
            self.winner = selfi.winner;

            if !self.winner {
                if self.stage != selfi.stage {
                    self.logs.add(turn, ClientLog::NextStage);
                    self.clear();
                }
                self.stage = selfi.stage;
            }
        }
        match diff {
            i if self.history.len() > i as usize => {
                let index = self.history.len() - i as usize - 1;
                self.history[index].info = Some(info);
                self.rebuild_current_state();
            }
            _ => {
                // Event too old, drop it.
                warn!("Dropping info because it is too old");
            }
        }

        if self.current_state.incoherent {
            self.clear();
            self.logs.add(turn, ClientLog::Lost);
        }
    }

    /// Resets the world. Mostly after a respawn or a goal reached.
    pub fn clear(&mut self) {
        self.history = vec![];
        self.past_state = WorldState::new();
        self.current_state = WorldState::new();
    }

    pub fn current_state(&self) -> &WorldState {
        &self.current_state
    }

    pub fn last_info(&self) -> SenseInfo {
        self.history
            .last()
            .and_then(|h| h.info.clone())
            .unwrap_or_default()
    }

    fn rebuild_current_state(&mut self) {
        // Should we take into account more recent terrain info ? It is static after all
        let mut state = self.past_state.clone();
        for history in self.history.iter() {
            state.update(history);
        }

        self.current_state = state;
    }
}

#[derive(Debug, Clone)]
struct WorldHistory {
    action: Action,
    senses: Senses,
    info: Option<SenseInfo>,
}

#[derive(Debug, Clone)]
pub struct WorldState {
    pub tiles: [Tile; VIEW_SIZE * VIEW_SIZE],
    pub broken: bool,
    pub signal: usize,
    pub position: Position,
    pub incoherent: bool,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            tiles: [Tile::Unknown; VIEW_SIZE * VIEW_SIZE],
            broken: false,
            signal: 100,
            position: START_POS,
            incoherent: false,
        }
    }

    pub fn tile_from_viewer(&self, offset: Offset) -> Tile {
        let position = self.position;
        if position.is_oob(VIEW_SIZE, VIEW_SIZE, offset) {
            Tile::Unknown
        } else {
            let pos = position + offset;
            self.tile_at(pos)
        }
    }

    pub fn tile_at(&self, pos: Position) -> Tile {
        let i = pos.x + VIEW_SIZE * pos.y;
        if i >= self.tiles.len() {
            return Tile::Unknown;
        }
        self.tiles[pos.x + VIEW_SIZE * pos.y]
    }

    fn update(&mut self, history: &WorldHistory) {
        self.update_action(&history.action);

        if let Some(ref info) = history.info {
            self.update_info(info);
        }

        self.apply_pylon_effect();
        let cost = history.senses.signal_cost();
        if self.signal >= cost {
            self.signal -= cost;
        }
    }

    fn update_action(&mut self, action: &Action) {
        match action {
            Action::Move(dir) => {
                if !self.broken {
                    let new_pos = self.position + dir.offset();
                    let tile = self.tile_at(new_pos);
                    if tile.can_travel() {
                        self.position = new_pos;
                    }
                }
            }
            Action::Spawn => {
                // Spawning actually cleans up the state
            }
            Action::Wait => {}
        }
    }

    fn apply_pylon_effect(&mut self) {
        for x in -1..2 {
            for y in -1..2 {
                let offset = Offset { x, y };
                let position = self.position + offset;
                let tile = self.tile_at(position);
                if matches!(tile, Tile::Pylon) {
                    self.signal = 100;
                }
            }
        }
    }

    fn update_info(&mut self, info: &SenseInfo) {
        if let Some(ref terrain) = info.terrain {
            let center = Position {
                x: terrain.radius,
                y: terrain.radius,
            };

            let iradius = terrain.radius as isize;
            let terrain_size = 2 * terrain.radius + 1;

            for x in (-iradius)..(iradius + 1) {
                for y in (-iradius)..(iradius + 1) {
                    let offset = Offset { x, y };
                    let info_pos = center + offset;
                    let tile = terrain.tiles[info_pos.as_index(terrain_size)];

                    if !matches!(tile, Tile::Unknown) {
                        let world_pos = self.position + offset;
                        let index = world_pos.as_index(VIEW_SIZE);
                        let old_tile = self.tiles[index];
                        if old_tile != Tile::Unknown && old_tile != tile {
                            self.incoherent = true;
                        }
                        self.tiles[index] = tile;
                    }
                }
            }
        }

        if let Some(ref selfs) = info.selfi {
            self.broken = selfs.broken;
            self.signal = selfs.signal;
        }
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}
