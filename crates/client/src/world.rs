use log::warn;
use losig_core::{
    network::TurnResultMessage,
    sense::SensesInfo,
    types::{ClientAction, Offset, Position, ServerAction, Tile, Tiles, Turn},
};

use crate::logs::{ClientLog, GameLogs};

const VIEW_SIZE: usize = 256;
const START_POS: Position = Position {
    x: VIEW_SIZE / 2,
    y: VIEW_SIZE / 2,
};

#[derive(Debug, Clone)]
pub struct WorldView {
    pub winner: bool,
    pub stage: u8,
    pub turn: Turn,

    history: Vec<WorldHistory>,
    past_state: WorldState,
    pub current_state: WorldState,
    pub logs: GameLogs,
    pub stage_turn: Turn,
}

impl WorldView {
    pub fn new() -> Self {
        let mut logs = GameLogs::default();
        logs.add(1, ClientLog::Help);

        Self {
            winner: false,
            stage: 0,
            turn: 1,
            stage_turn: 1,
            history: vec![],
            past_state: WorldState::default(),
            current_state: WorldState::default(),
            logs,
        }
    }

    pub fn act(&mut self, action: &ClientAction) {
        if matches!(action, ClientAction::Spawn) {
            self.clear();
        }

        let history = WorldHistory {
            action: *action,
            server_action: None,
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

    pub fn update(&mut self, turn_result: TurnResultMessage) {
        let TurnResultMessage {
            avatar_id: _,
            stage_turn,
            turn,
            stage,
            info,
            action,
            logs,
        } = turn_result;

        let diff = turn.abs_diff(self.turn);
        let turn_diff = (turn as i64) - (stage_turn as i64);

        // Update global info
        if diff == 0 {
            if self.stage != stage {
                self.clear();
            }
            self.stage = stage;
        }

        // Merge server logs
        self.logs.merge(logs, self.turn, turn_diff);
        match diff {
            i if self.history.len() > i as usize => {
                let index = self.history.len() - i as usize - 1;
                self.history[index].info = Some(info);
                self.history[index].server_action = Some(action);
                self.rebuild_current_state();
            }
            _ => {
                // Event too old, drop it.
                warn!("Dropping info because it is too old");
            }
        }

        self.stage_turn = stage_turn;
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

    pub fn last_info(&self) -> Option<&SensesInfo> {
        self.history.last().and_then(|h| h.info.as_ref())
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

impl Default for WorldView {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct WorldHistory {
    action: ClientAction,
    server_action: Option<ServerAction>,
    info: Option<SensesInfo>,
}

#[derive(Debug, Clone)]
pub struct WorldState {
    pub tiles: [Tile; VIEW_SIZE * VIEW_SIZE],
    pub position: Position,
    pub incoherent: bool,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            tiles: [Tile::Unknown; VIEW_SIZE * VIEW_SIZE],
            position: START_POS,
            incoherent: false,
        }
    }

    pub fn tile_from_viewer(&self, offset: Offset) -> Tile {
        let position = self.position + offset;
        self.tile_at(position)
    }

    pub fn tile_at(&self, pos: Position) -> Tile {
        let i = pos.x + VIEW_SIZE * pos.y;
        if i >= self.tiles.len() {
            return Tile::Unknown;
        }
        self.tiles[pos.x + VIEW_SIZE * pos.y]
    }

    fn update(&mut self, history: &WorldHistory) {
        self.update_action(&history.action, history.server_action.as_ref());

        if let Some(ref info) = history.info {
            if let Some(ref info) = info.sight {
                self.update_tiles(self.position, &info.tiles);
            }

            if let Some(ref info) = info.touch {
                self.update_tiles(self.position, &info.tiles);
            }
        }
    }

    fn update_action(&mut self, action: &ClientAction, server_action: Option<&ServerAction>) {
        match action {
            ClientAction::Spawn => {
                // Spawning actually cleans up the state
            }
            ClientAction::MoveOrAttack(dir) => {
                if matches!(server_action, None | Some(ServerAction::Move(_))) {
                    let new_pos = self.position + dir.offset();
                    let tile = self.tile_at(new_pos);
                    if tile.can_travel() {
                        self.position = new_pos;
                    }
                }
            }
            ClientAction::Wait => {}
        }
    }

    fn update_tiles(&mut self, viewer: Position, tiles: &Tiles) {
        let center = tiles.center();

        for ((src_x, src_y), &tile) in tiles.grid.indexed_iter() {
            if tile == Tile::Unknown {
                continue;
            }

            let world_x = (src_x as i32) - (center.x as i32) + (viewer.x as i32);
            let world_y = (src_y as i32) - (center.y as i32) + (viewer.y as i32);

            if world_x >= 0 && world_y >= 0 {
                let world_x = world_x as usize;
                let world_y = world_y as usize;

                if world_x < VIEW_SIZE && world_y < VIEW_SIZE {
                    let dest_i = world_x + world_y * VIEW_SIZE;
                    let dest_tile = self.tiles[dest_i];
                    if dest_tile != Tile::Unknown && tile != dest_tile {
                        self.incoherent = true;
                    }
                    self.tiles[dest_i] = tile;
                }
            }
        }
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}
