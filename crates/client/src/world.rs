use log::{debug, warn};
use losig_core::{
    fov,
    network::{TransitionMessage, TurnMessage},
    sense::{Senses, SensesInfo, SightInfo},
    types::{
        ClientAction, Offset, Position, ServerAction, StageId, StageTurn, Tile, Tiles, Timeline,
        Turn,
    },
};
use web_time::{Duration, Instant};

use crate::logs::{ClientLog, GameLogs};

const VIEW_SIZE: usize = 256;
const START_POS: Position = Position {
    x: VIEW_SIZE / 2,
    y: VIEW_SIZE / 2,
};

#[derive(Debug, Clone)]
pub struct WorldView {
    pub winner: bool,
    pub stage: StageId,
    pub turn: Turn,

    history: Vec<WorldDiff>,
    past_state: WorldState,
    pub current_state: WorldState,
    pub logs: GameLogs,
    pub stage_turn: StageTurn,
    pub timeline: Timeline,
    pub last_latency: Option<Duration>,
    action_sent_at: Option<Instant>,
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
            timeline: Timeline { head: 1, tail: 1 },
            last_latency: None,
            action_sent_at: None,
        }
    }

    pub fn act(&mut self, action: &ClientAction, senses: &Senses) {
        if matches!(action, ClientAction::Spawn) {
            self.clear();
        }

        // Record timestamp when action is sent
        self.action_sent_at = Some(Instant::now());

        let previous_info = self.last_info();
        let intermediate_info = WorldState::generate_intermediate_info(
            action,
            senses,
            &self.current_state,
            previous_info,
        );

        let history = WorldDiff {
            action: *action,
            update_received: false,
            server_action: None,
            info: intermediate_info,
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

    pub fn update(
        &mut self,
        TurnMessage {
            player_id: _,
            turn,
            stage_turn,
            stage,
            info,
            action,
            events,
            timeline,
        }: TurnMessage,
    ) {
        let diff = turn.abs_diff(self.turn);

        // Calculate latency if this is a response to our action
        if diff == 0
            && let Some(sent_at) = self.action_sent_at.take()
        {
            self.last_latency = Some(sent_at.elapsed());
        }

        // Update global info
        if diff == 0 {
            if self.stage != stage {
                self.clear();
            }
            self.stage = stage;
        }

        self.logs.add_server_events(turn, events);

        match diff {
            i if self.history.len() > i as usize => {
                let index = self.history.len() - i as usize - 1;
                self.history[index].info = info;
                self.history[index].server_action = Some(action);
                self.history[index].update_received = true;
                self.rebuild_current_state();
            }
            _ => {
                // Event too old, drop it.
                warn!("Dropping info because it is too old");
            }
        }

        self.stage_turn = stage_turn;
        self.timeline = timeline;
    }

    pub fn transition(
        &mut self,
        TransitionMessage {
            player_id: _,
            turn: _,
            stage_turn,
            stage,
            info,
            timeline,
        }: TransitionMessage,
    ) {
        self.clear();

        self.stage = stage;
        self.stage_turn = stage_turn;
        self.history.push(WorldDiff {
            action: ClientAction::Wait,
            server_action: Some(ServerAction::Wait),
            info,
            update_received: true,
        });
        self.timeline = timeline;
        self.rebuild_current_state();
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

    pub fn update_on_averted(&mut self, info: SensesInfo) {
        // Add the averted info to the last history entry if it exists
        if let Some(last) = self.history.last_mut() {
            last.info = Some(info);
            self.rebuild_current_state();
        }
    }

    pub fn update_on_timeline(&mut self, stage_turn: StageTurn, info: SensesInfo) {
        // Calculate turn from stage_turn
        let turn_diff = (self.turn as i64) - (self.stage_turn as i64);
        let turn = (stage_turn as i64 + turn_diff) as u64;
        let diff = self.turn.abs_diff(turn);

        // Update the history entry at the calculated position
        if self.history.len() > diff as usize {
            let index = self.history.len() - diff as usize - 1;
            self.history[index].info = Some(info);
            self.rebuild_current_state();
        }
    }

    fn rebuild_current_state(&mut self) {
        let mut state = self.past_state.clone();
        for history in self.history.iter() {
            state.update(history);
        }

        debug!("Rebuilding state up to turn {}", self.turn);
        self.current_state = state;
    }

    pub fn update_timeline(&mut self, stage: StageId, timeline: Timeline) {
        if self.stage == stage {
            self.timeline = timeline;
        }
    }
}

impl Default for WorldView {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct WorldDiff {
    action: ClientAction,
    update_received: bool,
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

    fn update(&mut self, history: &WorldDiff) {
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

    /// Convert client's tile array to Tiles struct for FOV calculation
    fn tiles_for_fov(&self, radius: usize) -> Tiles {
        let size = 2 * radius + 1;
        let mut tiles = Tiles::new(size, size);
        let center_offset = radius as isize;

        for x in 0..size {
            for y in 0..size {
                let world_x = (self.position.x as isize) + (x as isize) - center_offset;
                let world_y = (self.position.y as isize) + (y as isize) - center_offset;

                if world_x >= 0 && world_y >= 0 {
                    let world_x = world_x as usize;
                    let world_y = world_y as usize;

                    if world_x < VIEW_SIZE && world_y < VIEW_SIZE {
                        let tile_index = world_x + world_y * VIEW_SIZE;
                        tiles.grid[(x, y)] = self.tiles[tile_index];
                    }
                }
            }
        }

        tiles
    }
    /// Generate intermediate sense info using client-side FOV to prevent flickers.
    /// This shows entities at their last known positions "as if they didn't move"
    /// while waiting for the server response.
    fn generate_intermediate_info(
        action: &ClientAction,
        senses: &Senses,
        current_state: &WorldState,
        previous_info: Option<&SensesInfo>,
    ) -> Option<SensesInfo> {
        // Predict the state after this action to get the right position
        let old_position = current_state.position;
        let mut predicted_state = current_state.clone();
        predicted_state.update_action(action, None);
        let new_position = predicted_state.position;

        // Calculate player movement offset to adjust entity offsets
        let player_movement = new_position - old_position;

        // Generate sight info if sight sense is active
        let sight = if senses.sight.get() > 0 {
            let previous_sight = previous_info.and_then(|info| info.sight.as_ref());
            let previous_sight_radius = previous_sight
                .map(|sight| (sight.tiles.width().saturating_sub(1)) / 2)
                .unwrap_or(0);

            let requested_radius = senses.sight.get() as usize;
            let sight_radius = requested_radius.min(previous_sight_radius);

            if sight_radius > 0 {
                // Convert client tiles to Tiles for FOV calculation
                let tiles_for_fov = predicted_state.tiles_for_fov(sight_radius);

                // Calculate FOV from center of the local grid
                let center_pos = Position {
                    x: sight_radius,
                    y: sight_radius,
                };
                let fov_tiles = fov::fov(center_pos, sight_radius, &tiles_for_fov);

                // Copy foes, orb, and allies from previous sight, adjusting offsets for player movement
                let (foes, orb, allies) = if let Some(prev_sight) = previous_sight {
                    let adjusted_foes = prev_sight
                        .foes
                        .iter()
                        .map(|foe| {
                            let mut adjusted = foe.clone();
                            adjusted.offset = adjusted.offset - player_movement;
                            adjusted
                        })
                        .collect();

                    let adjusted_orb = prev_sight.orb.map(|offset| offset - player_movement);

                    let adjusted_allies = prev_sight
                        .allies
                        .iter()
                        .map(|ally| {
                            let mut adjusted = ally.clone();
                            adjusted.offset = adjusted.offset - player_movement;
                            // Also adjust the offset in Leading status if present
                            if let Some(offset) = adjusted.next_move.as_mut() {
                                *offset = *offset - player_movement;
                            }
                            adjusted
                        })
                        .collect();

                    (adjusted_foes, adjusted_orb, adjusted_allies)
                } else {
                    (vec![], None, vec![])
                };

                Some(SightInfo {
                    tiles: fov_tiles,
                    foes,
                    orb,
                    allies,
                })
            } else {
                None
            }
        } else {
            None
        };

        // Copy other sense infos from previous state
        let selfi = previous_info.and_then(|info| info.selfi.clone());
        let touch = previous_info.and_then(|info| info.touch.clone());
        let hearing = previous_info.and_then(|info| info.hearing.clone());

        // Return intermediate info if at least one sense is present
        if sight.is_some() || selfi.is_some() || touch.is_some() || hearing.is_some() {
            Some(SensesInfo {
                selfi,
                touch,
                sight,
                hearing,
            })
        } else {
            None
        }
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}
