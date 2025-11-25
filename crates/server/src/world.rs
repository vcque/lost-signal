use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Bound,
};

use anyhow::{Result, anyhow};
use grid::Grid;
use log::{debug, info, warn};
use losig_core::{
    sense::{Senses, SensesInfo},
    types::{Action, Avatar, AvatarId, Foe, GameOver, Offset, Position, Tile, Tiles, Turn},
};

use crate::{fov, sense::gather};

#[derive(Debug, Clone)]
pub struct World {
    pub stages: Vec<Stage>,
}

impl World {
    pub fn new(stages: Vec<Stage>) -> World {
        World { stages }
    }
}

#[derive(Debug, Clone)]
pub struct Stage {
    pub tiles: Tiles,
    pub orb_spawns: Grid<bool>,
    pub foes: Vec<Foe>,
    pub orb: Position,
}

impl Stage {
    pub fn new(tiles: Tiles, orb_spawns: Grid<bool>, foes: Vec<Foe>) -> Self {
        let mut new = Self {
            tiles,
            foes,
            orb_spawns,
            orb: Position::default(),
        };

        new.move_orb();
        new
    }

    pub fn move_orb(&mut self) {
        self.orb = orb_spawn(&self.orb_spawns);
    }
}

pub struct AsyncWorld {
    pub stage_id_by_avatar_id: BTreeMap<AvatarId, usize>,
    pub stages: Vec<AsyncStage>,
}

impl AsyncWorld {
    pub fn new(stages: Vec<Stage>) -> Self {
        AsyncWorld {
            stage_id_by_avatar_id: Default::default(),
            stages: stages.into_iter().map(AsyncStage::new).collect(),
        }
    }

    pub fn create_avatar(&mut self, aid: AvatarId) {
        info!("Create a new avatar #{aid}");
        self.stage_id_by_avatar_id.insert(aid, 0);
        let stage = &mut self.stages[0];

        let avatar = Avatar {
            id: aid,
            stage: 0,
            position: Default::default(),
            focus: 100,
            turns: 0,
            tired: false,
            gameover: None,
        };

        stage.add_avatar(avatar);
    }

    pub fn retire_avatar(&mut self, _aid: AvatarId) -> Option<GameOver> {
        None
    }

    pub fn add_command(
        &mut self,
        aid: AvatarId,
        action: Action,
        senses: Senses,
    ) -> Option<SensesInfo> {
        let stage_id = *self.stage_id_by_avatar_id.get(&aid)?;
        let stage = &mut self.stages[stage_id];

        match stage.add_command(aid, action, senses) {
            Ok(info) => Some(info),
            Err(e) => {
                warn!("Error applying command: {e}");
                None
            }
        }
    }
}

/// Stage that can handle async actions from players
pub struct AsyncStage {
    /*
     * Static stage info
     */
    pub tiles: Tiles,
    pub orb_spawns: Grid<bool>,

    /*
     * Rollback handling
     */
    head_turn: Turn,
    avatar_turns: BTreeMap<AvatarId, Turn>,
    states: BTreeMap<Turn, StageState>,
    diffs: Vec<StageDiff>,
}

impl AsyncStage {
    pub fn new(
        Stage {
            tiles,
            orb_spawns,
            foes,
            orb,
        }: Stage,
    ) -> Self {
        let head_turn: Turn = 0;
        let avatars = vec![];
        let state = StageState { foes, orb, avatars };
        let mut states = BTreeMap::<Turn, StageState>::new();
        states.insert(head_turn, state);

        Self {
            tiles,
            orb_spawns,
            head_turn,
            avatar_turns: Default::default(),
            states,
            diffs: vec![StageDiff::default()],
        }
    }

    pub fn last_state(&self) -> &StageState {
        self.states.last_key_value().unwrap().1
    }

    fn add_avatar(&mut self, avatar: Avatar) {
        let aid = avatar.id;
        self.head_turn += 1;
        self.diffs.push(StageDiff {
            diff_by_avatar: Default::default(),
            new_avatar: Some(avatar),
        });

        self.avatar_turns.insert(aid, self.head_turn);
        self.rollback_from(self.head_turn);
    }
    fn add_command(&mut self, aid: u32, action: Action, senses: Senses) -> Result<SensesInfo> {
        let turn = self
            .avatar_turns
            .get(&aid)
            .ok_or_else(|| anyhow!("Could not find turn"))?;

        // Next turn
        let turn = turn + 1;
        self.avatar_turns.insert(aid, turn);

        // Add a new item to the list if at the head of the difflist
        if turn > self.head_turn {
            self.head_turn += 1;
            self.diffs.push(StageDiff {
                diff_by_avatar: Default::default(),
                new_avatar: None,
            });
        }

        // Add the diff to the turn
        let turn_diff = self.head_turn - turn;
        let index = self.diffs.len() - 1 - turn_diff as usize;

        let avatar_diff = StageAvatarDiff { action, senses };
        self.diffs
            .get_mut(index)
            .ok_or_else(|| anyhow!("Incoherent difflist: no index {index}"))?
            .diff_by_avatar
            .insert(aid, avatar_diff);

        self.rollback_from(turn);
        self.clean_history();

        // Apply senses
        self.gather_info(aid)
    }

    /// Recomputes the states from the given turn and forward
    fn rollback_from(&mut self, turn: Turn) -> Option<()> {
        debug!("Rolling back from {turn}");
        // Get the previous state available
        let (turn, state) = self
            .states
            .range((Bound::Unbounded, Bound::Included(turn)))
            .next_back()?;

        let turn = *turn;
        debug!("found state at turn {turn}");
        let mut state = state.clone();

        let mut turns_to_save = self.avatar_turns.values().copied().collect::<BTreeSet<_>>();
        turns_to_save.insert(self.head_turn);
        debug!("Avatars turns: {:?}", self.avatar_turns);
        debug!("Turns to save {turns_to_save:?}");
        self.states.retain(|key, _| *key <= turn);

        for turn in (turn + 1)..(self.head_turn + 1) {
            let index = self.diff_index(turn);
            let diff = &self.diffs[index];
            self.update(&mut state, diff);
            if turns_to_save.contains(&turn) {
                debug!("Saving state at turn {turn}");
                self.states.insert(turn, state.clone());
            }
        }
        Some(())
    }

    fn diff_index(&self, turn: Turn) -> usize {
        let turn_diff = self.head_turn - turn;
        self.diffs.len() - 1 - turn_diff as usize
    }

    /// Remove old states that are no more used: e.g. turns older than the earliest avatar turn
    fn clean_history(&mut self) {
        let Some(oldest_turn) = self.avatar_turns.values().min() else {
            return;
        };

        let index = self.diff_index(*oldest_turn);
        self.diffs.drain(0..index);
    }
    fn gather_info(&self, aid: AvatarId) -> Result<SensesInfo> {
        // 1. retrieve turn
        let turn = self
            .avatar_turns
            .get(&aid)
            .ok_or_else(|| anyhow!("Could not find turn"))?;

        // 2. retrieve state
        let mut state = self
            .states
            .get(turn)
            .ok_or_else(|| anyhow!("Cound not find state"))?
            .clone();

        // 3. apply actions of the next turn for avatars (not foes)
        let diff_index = self.diff_index(*turn);
        if let Some(next_diff) = self.diffs.get(diff_index + 1) {
            self.update_commands(&mut state, next_diff);
        }

        // 4. use the senses for this
        let senses = self.diffs[diff_index]
            .diff_by_avatar
            .get(&aid)
            .map(|d| &d.senses);

        if let Some(senses) = senses {
            let (avatar, stage) = state.into_sync(self, aid);
            if !avatar.tired {
                return Ok(gather(senses, &avatar, &stage));
            }
        }
        Ok(Default::default())
    }

    /// Update a state based on the diff
    fn update(&self, state: &mut StageState, diff: &StageDiff) {
        self.update_commands(state, diff);
        self.update_foes(state);

        if let Some(ref new_avatar) = diff.new_avatar {
            self.welcome_avatar(state, new_avatar);
        }
    }

    /// Apply the turn of each avatar
    fn update_commands(&self, state: &mut StageState, diff: &StageDiff) {
        for (aid, StageAvatarDiff { action, senses }) in diff.diff_by_avatar.iter() {
            let Some(avatar) = state.avatars.iter_mut().find(|a| a.id == *aid) else {
                continue;
            };

            match action {
                Action::Move(dir) => {
                    let next_pos = avatar.position.move_once(*dir);

                    let tile = self.tiles.grid[next_pos.into()];
                    if tile.can_travel() {
                        avatar.position = next_pos;
                    }
                }
                Action::Spawn => {}
                _ => {}
            }

            // Orb on tile
            if avatar.position == state.orb {
                // TODO: move orb
            }

            // Sense cost
            let cost = senses.cost();
            avatar.tired = avatar.focus <= cost;
            if !avatar.tired {
                avatar.focus -= cost;
            }

            // Orb in sight
            if fov::can_see(&self.tiles, avatar.position, state.orb) {
                // TODO: excite orb -> move next turn
            }

            // If pylon is adjacent, recharges focus
            for x in -1..2 {
                for y in -1..2 {
                    let offset = Offset { x, y };
                    let position = avatar.position + offset;
                    let tile = self.tiles.grid[position.into()];
                    if matches!(tile, Tile::Pylon) {
                        avatar.focus = 100;
                    }
                }
            } // Pylon effect
        }
    }

    /// Apply the turn of each foe
    fn update_foes(&self, state: &mut StageState) {
        // Foes are static for now
        for foe in state.foes.iter() {
            for avatar in state.avatars.iter_mut() {
                if foe.position == avatar.position {
                    avatar.gameover = Some(GameOver::new(avatar, false));
                }
            }
        }
    }

    /// Update the world to spawn the user
    fn welcome_avatar(&self, state: &mut StageState, avatar: &Avatar) {
        let aid = avatar.id;
        let spawn_position = self.find_spawns();
        let position = spawn_position[aid as usize % spawn_position.len()];

        let mut avatar = avatar.clone();
        avatar.position = position;
        state.avatars.push(avatar);
    }

    pub fn find_spawns(&self) -> Vec<Position> {
        self.tiles
            .grid
            .indexed_iter()
            .filter_map(|((x, y), t)| {
                if *t == Tile::Spawn {
                    Some(Position { x, y })
                } else {
                    None
                }
            })
            .collect()
    }
}

/// State of a stage for a given turn.
#[derive(Clone)]
pub struct StageState {
    pub foes: Vec<Foe>,
    pub orb: Position,
    pub avatars: Vec<Avatar>,
}

impl StageState {
    /// Convert to sync. Is not ideal as it clones the states. Will change once the sync version is
    /// removed
    fn into_sync(self, stage: &AsyncStage, aid: AvatarId) -> (Avatar, Stage) {
        let Self { foes, orb, avatars } = self;
        let mut stage = Stage::new(stage.tiles.clone(), stage.orb_spawns.clone(), foes);
        stage.orb = orb;

        let avatar = avatars.into_iter().find(|a| a.id == aid).unwrap();
        (avatar, stage)
    }
}

/// What's needed to recompute a stage state
#[derive(Clone, Default)]
struct StageDiff {
    diff_by_avatar: BTreeMap<AvatarId, StageAvatarDiff>,
    new_avatar: Option<Avatar>,
}

#[derive(Clone)]
struct StageAvatarDiff {
    action: Action,
    senses: Senses,
    // TODO: should we also save info gathered?
}

// TODO: make it deterministic
fn orb_spawn(spawns: &Grid<bool>) -> Position {
    let spawns: Vec<Position> = spawns
        .indexed_iter()
        .filter(|(_, val)| **val)
        .map(|(pos, _)| Position::from(pos))
        .collect();

    if spawns.is_empty() {
        warn!("Couldn't find a spawn point for lvl");
        return Default::default();
    }
    let i = rand::random_range(0..spawns.len());
    spawns[i]
}
