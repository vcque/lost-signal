use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Bound,
};

use anyhow::{Result, anyhow};
use grid::Grid;
use log::{debug, info, warn};
use losig_core::{
    sense::{Senses, SensesInfo},
    types::{
        Action, Avatar, AvatarId, Foe, GameOver, GameOverStatus, Offset, Position, Tile, Tiles,
        Turn,
    },
};

use crate::{foes, fov, sense::gather};

/// Data of a stage that can not change with time or action players
#[derive(Debug, Clone)]
pub struct StageTemplate {
    pub id: String,
    pub tiles: Tiles,
    pub orb_spawns: Grid<bool>,
    pub foes: Vec<Foe>,
}

impl StageTemplate {
    pub fn new(id: String, tiles: Tiles, orb_spawns: Grid<bool>, foes: Vec<Foe>) -> Self {
        Self {
            id,
            tiles,
            foes,
            orb_spawns,
        }
    }
}

pub struct World {
    pub stage_id_by_avatar_id: BTreeMap<AvatarId, usize>,
    pub stages: Vec<Stage>,
}

impl World {
    pub fn new(stages: Vec<StageTemplate>) -> Self {
        World {
            stage_id_by_avatar_id: Default::default(),
            stages: stages.into_iter().map(Stage::new).collect(),
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
            hp: 10,
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
    ) -> Option<CommandResult> {
        let stage_id = *self.stage_id_by_avatar_id.get(&aid)?;
        let stage = &mut self.stages[stage_id];

        match stage.add_command(aid, action, senses) {
            Ok(result) => Some(result),
            Err(e) => {
                warn!("Error applying command: {e}");
                None
            }
        }
    }
}

/// Stage that can handle async actions from players
pub struct Stage {
    /*
     * Static stage info
     */
    pub template: StageTemplate,

    /*
     * Rollback handling
     */
    head_turn: Turn,
    avatar_turns: BTreeMap<AvatarId, Turn>,
    states: BTreeMap<Turn, StageState>,
    diffs: Vec<StageDiff>,

    /// Tracker of maybe dead avatars. We need to notify the player when they revive... Or not
    limbo: BTreeSet<AvatarId>,
}

impl Stage {
    pub fn new(stage: StageTemplate) -> Self {
        let head_turn: Turn = 0;
        let avatars = Default::default();

        let state = StageState {
            foes: stage.foes.clone(),
            orb: orb_spawn(&stage.orb_spawns),
            avatars,
        };
        let mut states = BTreeMap::<Turn, StageState>::new();
        states.insert(head_turn, state);

        Self {
            template: stage,
            head_turn,
            avatar_turns: Default::default(),
            states,
            limbo: Default::default(),
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

    fn add_command(&mut self, aid: u32, action: Action, senses: Senses) -> Result<CommandResult> {
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

        self.rollback_from(turn - 1);
        self.clean_history();

        let (gameovers, gameovers_reverted) = self.check_limbo();

        // Apply senses
        let senses_info = self.gather_info(aid)?;

        Ok(CommandResult {
            gameovers,
            gameovers_reverted,
            senses_info,
        })
    }

    /// Recomputes the states from the given turn and forward
    fn rollback_from(&mut self, turn: Turn) -> Option<()> {
        // Get the previous state available
        let (turn, state) = self
            .states
            .range((Bound::Unbounded, Bound::Included(turn)))
            .next_back()?;

        let turn = *turn;
        let mut state = state.clone();

        let mut turns_to_save = self.avatar_turns.values().copied().collect::<BTreeSet<_>>();
        turns_to_save.insert(self.head_turn);
        self.states.retain(|key, _| *key <= turn);

        for turn in (turn + 1)..(self.head_turn + 1) {
            let index = self.diff_index(turn);
            let diff = &self.diffs[index];
            self.update(&mut state, diff);
            if turns_to_save.contains(&turn) {
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

        debug!("Gathering info for {aid} at turn {turn}");

        // 3. apply actions of the next turn for avatars (not foes)
        let diff_index = self.diff_index(*turn);
        if let Some(next_diff) = self.diffs.get(diff_index + 1) {
            debug!("Applying diff {next_diff:?}");
            self.update_commands(&mut state, next_diff);
        }

        // 4. use the senses for this
        let senses = self.diffs[diff_index]
            .diff_by_avatar
            .get(&aid)
            .map(|d| &d.senses);

        if let Some(senses) = senses {
            let avatar = &state.avatars[&aid];
            if !avatar.tired {
                return Ok(gather(senses, avatar, self, &state));
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
            let Some(avatar) = state.avatars.get(aid) else {
                continue;
            };

            if avatar.is_dead() {
                continue;
            }

            let mut avatar = avatar.clone();

            match action {
                Action::Move(dir) => {
                    let next_pos = avatar.position.move_once(*dir);

                    let tile = self.template.tiles.grid[next_pos.into()];

                    // temporary code for simulating battle
                    let mut attack = false;
                    if let Some(Foe::Simple(_, hp)) = state.find_foe(next_pos) {
                        *hp -= 1;
                        attack = true;
                    }
                    state.foes.retain(|f| f.alive());

                    if !attack && tile.can_travel() {
                        avatar.position = next_pos;
                    }
                }
                Action::Spawn => {
                    let spawn_position = self.find_spawns();
                    avatar.position = spawn_position[avatar.id as usize % spawn_position.len()];
                    avatar.hp = 10;
                    avatar.focus = 100;
                }
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
            if fov::can_see(&self.template.tiles, avatar.position, state.orb) {
                // TODO: excite orb -> move next turn
            }

            // If pylon is adjacent, recharges focus
            for x in -1..2 {
                for y in -1..2 {
                    let offset = Offset { x, y };
                    let position = avatar.position + offset;
                    let tile = self.template.tiles.grid[position.into()];
                    if matches!(tile, Tile::Pylon) {
                        avatar.focus = 100;
                    }
                }
            } // Pylon effect

            state.avatars.insert(*aid, avatar);
        }
    }

    /// Apply the turn of each foe
    fn update_foes(&self, state: &mut StageState) {
        // Foes are static for now
        for i in 0..state.foes.len() {
            let foe = state.foes[i].clone();
            let mutator = foes::act(&foe, self, state);
            mutator(&mut state.foes[i]);
        }
    }

    /// Update the world to spawn the user
    fn welcome_avatar(&self, state: &mut StageState, avatar: &Avatar) {
        let aid = avatar.id;
        let spawn_position = self.find_spawns();
        let position = spawn_position[aid as usize % spawn_position.len()];

        let mut avatar = avatar.clone();
        avatar.position = position;
        state.avatars.insert(aid, avatar);
    }

    pub fn find_spawns(&self) -> Vec<Position> {
        self.template
            .tiles
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

    /// Returns deaths, maybedeaths and reverted deaths
    fn check_limbo(&mut self) -> (Vec<(AvatarId, GameOver)>, Vec<AvatarId>) {
        let mut gameovers = Vec::new();
        let mut gameovers_reverted = Vec::new();

        // Sort avatars by turn (earliest to latest)
        let mut avatars_by_turn: Vec<_> = self.avatar_turns.iter().map(|(a, b)| (*a, *b)).collect();
        avatars_by_turn.sort_by_key(|(_, turn)| *turn);

        let mut has_earlier_alive = false;
        for (aid, turn) in avatars_by_turn {
            let Some(state) = self.states.get(&turn) else {
                continue;
            };

            let Some(avatar) = state.avatars.get(&aid) else {
                continue;
            };

            let in_limbo = self.limbo.contains(&aid);

            if avatar.is_dead() {
                if has_earlier_alive {
                    // Uncertain death - earlier avatars might save them
                    self.limbo.insert(aid);
                    if !in_limbo {
                        gameovers.push((aid, GameOver::new(avatar, GameOverStatus::MaybeDead)));
                    }
                } else {
                    // Can't be saved
                    self.limbo.remove(&aid);
                    self.avatar_turns.remove(&aid);
                    gameovers.push((aid, GameOver::new(avatar, GameOverStatus::Dead)));
                }
            } else {
                if in_limbo {
                    // Avatar is alive and was in limbo - death was reverted
                    self.limbo.remove(&aid);
                    gameovers_reverted.push(aid);
                }
                has_earlier_alive = true;
            }
        }

        (gameovers, gameovers_reverted)
    }
}

/// State of a stage for a given turn.
#[derive(Clone)]
pub struct StageState {
    pub foes: Vec<Foe>,
    pub orb: Position,
    pub avatars: BTreeMap<AvatarId, Avatar>,
}

impl StageState {
    fn find_foe(&mut self, position: Position) -> Option<&mut Foe> {
        self.foes.iter_mut().find(|f| f.position() == position)
    }
}

/// What's needed to recompute a stage state
#[derive(Clone, Default, Debug)]
struct StageDiff {
    diff_by_avatar: BTreeMap<AvatarId, StageAvatarDiff>,
    new_avatar: Option<Avatar>,
}

#[derive(Clone, Debug)]
struct StageAvatarDiff {
    action: Action,
    senses: Senses,
}

/// Info returned by add_command. Game over data might concern other players as they can be saved
/// by another player.
pub struct CommandResult {
    pub gameovers: Vec<(AvatarId, GameOver)>,
    pub gameovers_reverted: Vec<AvatarId>,
    pub senses_info: SensesInfo,
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
