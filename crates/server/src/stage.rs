use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Bound,
};

use anyhow::{Result, anyhow};
use log::{info, warn};
use losig_core::{
    sense::{Senses, SensesInfo},
    types::{
        Avatar, AvatarId, ClientAction, Foe, GameLogEvent, Offset, Orb, Position, ServerAction,
        StageTurn, Tile, Transition, Turn,
    },
};

use crate::{
    action, foes, fov,
    sense::gather,
    world::{Limbo, StageTemplate},
};

/// Stage that can handle async actions from players
pub struct Stage {
    /*
     * Static stage info
     */
    pub template: StageTemplate,
    seed: u64,

    /*
     * Rollback handling
     */
    head_turn: Turn,
    avatar_trackers: BTreeMap<AvatarId, AvatarTracker>,
    states: BTreeMap<Turn, StageState>,
    diffs: Vec<StageDiff>,
}

impl Stage {
    pub fn new(stage: StageTemplate) -> Self {
        let head_turn: Turn = 0;
        let avatars = Default::default();
        let seed: u64 = rand::random();

        let mut new = Self {
            template: stage,
            seed,
            head_turn,
            avatar_trackers: Default::default(),
            states: Default::default(),
            diffs: vec![StageDiff::default()],
        };

        let state = StageState {
            turn: head_turn,
            foes: new.template.foes.clone(),
            orb: Orb {
                position: orb_spawn(&new, head_turn),
                excited: false,
            },
            avatars,
        };
        new.states.insert(head_turn, state);
        new
    }

    pub fn head_state(&self) -> &StageState {
        self.states.last_key_value().unwrap().1
    }

    pub fn state_for(&self, aid: AvatarId) -> Option<StageState> {
        let tracker = self.avatar_trackers.get(&aid)?;
        Some(self.states.get(&tracker.turn)?.clone())
    }

    pub fn add_avatar(&mut self, avatar: Avatar) {
        let aid = avatar.id;
        self.head_turn += 1;
        self.diffs.push(StageDiff {
            diff_by_avatar: Default::default(),
            new_avatar: Some(avatar),
        });

        self.avatar_trackers
            .insert(aid, AvatarTracker::new(self.head_turn));
        self.rollback_from(self.head_turn);
    }

    pub fn remove_avatar(&mut self, aid: AvatarId) -> Option<Avatar> {
        let state = self.state_for(aid)?;
        let avatar = state.avatars.get(&aid);

        self.avatar_trackers.remove(&aid);
        self.clean_history();
        avatar.cloned()
    }

    pub fn add_command(
        &mut self,
        aid: AvatarId,
        action: ClientAction,
        senses: Senses,
    ) -> Result<StageCommandResult> {
        let action = action::convert_client(action, self, aid);

        let tracker = self
            .avatar_trackers
            .get_mut(&aid)
            .ok_or_else(|| anyhow!("Could not find turn"))?;

        // Next turn
        tracker.turn += 1;

        // Add a new item to the list if at the head of the difflist
        if tracker.turn > self.head_turn {
            self.head_turn += 1;
            self.diffs.push(StageDiff {
                diff_by_avatar: Default::default(),
                new_avatar: None,
            });
        }

        let stage_turn = tracker.turn; // ends mutable borrow

        let diff_index = self.diff_index(stage_turn);

        let avatar_diff = StageAvatarDiff {
            action,
            senses: senses.clone(),
        };
        self.diffs
            .get_mut(diff_index)
            .ok_or_else(|| anyhow!("Incoherent difflist: no index {diff_index}"))?
            .diff_by_avatar
            .insert(aid, avatar_diff);

        self.rollback_from(stage_turn);

        // fetch state for aid
        let state = self
            .state_for(aid)
            .ok_or_else(|| anyhow!("Couldn't find state at {aid}"))?;
        let avatar = state
            .avatars
            .get(&aid)
            .cloned()
            .ok_or_else(|| anyhow!("Couldn't find avatar at {aid}"))?;

        let senses_info = if avatar.tired {
            SensesInfo::default()
        } else {
            gather(&senses, &avatar, self, &state)
        };
        let logs = avatar.logs;

        // Limbo handling
        let limbos = self.handle_limbo();

        self.clean_history();

        Ok(StageCommandResult {
            stage_turn,
            limbos,
            logs,
            senses_info,
            action,
            transition: avatar.transition,
        })
    }

    /// Recomputes the states from the given turn and forward
    fn rollback_from(&mut self, turn: Turn) -> Option<()> {
        // Get the previous state available
        let (turn, state) = self
            .states
            .range((Bound::Unbounded, Bound::Excluded(turn)))
            .next_back()?;

        let turn = *turn;
        let mut state = state.clone();

        let mut turns_to_save = self
            .avatar_trackers
            .values()
            .map(|tr| tr.turn)
            .collect::<BTreeSet<_>>();

        turns_to_save.insert(self.head_turn);
        self.states.retain(|key, _| *key <= turn);

        for turn in (turn + 1)..(self.head_turn + 1) {
            let index = self.diff_index(turn);
            let diff = &self.diffs[index];
            self.enact_turn(&mut state, diff);
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
        if let Some(oldest_turn) = self.avatar_trackers.values().map(|tr| tr.turn).min() {
            let index = self.diff_index(oldest_turn);
            self.diffs.drain(0..index);
        } else {
            info!(
                "Stage {} is reset because it has no more player",
                self.template.id
            );

            *self = Self::new(self.template.clone());
        }
    }

    fn gather_info(&self, aid: AvatarId, senses: &Senses) -> Result<SensesInfo> {
        let state = self
            .state_for(aid)
            .ok_or_else(|| anyhow!("Could not find state for {aid}"))?;

        let avatar = &state.avatars[&aid];
        if !avatar.tired {
            return Ok(gather(senses, avatar, self, &state));
        }
        Ok(Default::default())
    }

    /// Update a state based on the diff
    fn enact_turn(&self, state: &mut StageState, diff: &StageDiff) {
        // Turn init
        if state.orb.excited {
            state.orb.position = orb_spawn(self, state.turn);
            state.orb.excited = false;
        }

        self.enact_avatars(state, diff);
        self.enact_foes(state);

        if let Some(ref new_avatar) = diff.new_avatar {
            self.welcome_avatar(state, new_avatar);
        }
        state.turn += 1;
    }

    /// Apply the turn of each avatar
    fn enact_avatars(&self, state: &mut StageState, diff: &StageDiff) {
        for (
            aid,
            StageAvatarDiff {
                action: player_action,
                senses,
            },
        ) in diff.diff_by_avatar.iter()
        {
            let Some(avatar) = state.avatars.get(aid) else {
                continue;
            };

            if avatar.is_dead() {
                continue;
            }

            let mut avatar = avatar.clone();

            // Execute the action
            action::act(player_action, &mut avatar, state, self);

            // Orb on tile
            if avatar.position == state.orb.position {
                state.orb.position = orb_spawn(self, state.turn);
                avatar.transition = Some(Transition::Orb);
            }

            // Sense cost
            let cost = senses.cost();
            avatar.tired = avatar.focus < cost;
            if !avatar.tired {
                avatar.focus = avatar.focus.saturating_sub(cost);
            }

            // Orb in sight
            if !avatar.tired
                && fov::can_see(
                    &self.template.tiles,
                    avatar.position,
                    state.orb.position,
                    senses.sight.get(),
                )
            {
                state.orb.excited = true;
                avatar.logs.push((state.turn, GameLogEvent::OrbSeen));
            }

            // If pylon is adjacent, recharges focus
            for x in -1..2 {
                for y in -1..2 {
                    let offset = Offset { x, y };
                    let position = avatar.position + offset;
                    let tile = self.template.tiles.get(position);
                    if matches!(tile, Tile::Pylon) {
                        avatar.focus = 100;
                    }
                }
            }

            avatar.turns += 1;
            state.avatars.insert(*aid, avatar);
        }
    }

    /// Apply the turn of each foe
    fn enact_foes(&self, state: &mut StageState) {
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

    pub fn handle_limbo(&mut self) -> Vec<Limbo> {
        let statuses = self.limbo_check();
        for status in statuses.iter() {
            match status {
                Limbo::Dead(avatar) | Limbo::TooFarBehind(avatar) => {
                    self.avatar_trackers.remove(&avatar.id);
                }
                &Limbo::MaybeDead(aid) => {
                    self.avatar_trackers.get_mut(&aid).unwrap().limbo = true;
                }
                &Limbo::Averted(aid, _) => {
                    self.avatar_trackers.get_mut(&aid).unwrap().limbo = false;
                }
            }
        }

        statuses
    }

    /// Returns deaths, maybedeaths and reverted deaths
    fn limbo_check(&mut self) -> Vec<Limbo> {
        // Sort avatars by turn (earliest to latest)
        let mut aid_n_trackers: Vec<_> =
            self.avatar_trackers.iter().map(|(a, b)| (*a, b)).collect();
        aid_n_trackers.sort_by_key(|(_, tr)| tr.turn);

        let mut has_earlier_alive = false;
        let mut results = vec![];

        for (aid, tracker) in aid_n_trackers {
            let Some(state) = self.states.get(&tracker.turn) else {
                warn!("Tracker of {aid} has no state!");
                continue;
            };

            let Some(avatar) = state.avatars.get(&aid).cloned() else {
                warn!("State of {aid} tracker has no corresponding avatar!");
                continue;
            };

            if tracker.turn.abs_diff(self.head_turn) > 100 {
                results.push(Limbo::TooFarBehind(avatar));
                continue;
            }

            let in_limbo = tracker.limbo;
            let dead = avatar.is_dead();
            let status = match (has_earlier_alive, in_limbo, avatar.is_dead()) {
                (false, _, true) => Some(Limbo::Dead(avatar.clone())),
                (_, true, false) => {
                    // Get senses from the diff at this turn
                    let index = self.diff_index(tracker.turn);
                    let senses = self
                        .diffs
                        .get(index)
                        .and_then(|diff| diff.diff_by_avatar.get(&aid))
                        .map(|avatar_diff| &avatar_diff.senses)
                        .cloned()
                        .unwrap_or_default();

                    let senses_info = self.gather_info(aid, &senses).unwrap_or_default();
                    Some(Limbo::Averted(aid, senses_info))
                }
                (true, false, true) => Some(Limbo::MaybeDead(aid)),
                _ => None, // If state does not change, do not notify
            };

            if !dead {
                has_earlier_alive = true;
            }

            if let Some(status) = status {
                results.push(status);
            }
        }
        results
    }
}

#[derive(Clone)]
struct AvatarTracker {
    turn: Turn,
    /// Limbo means a message of MaybeDead has been sent to the player and is awaiting
    /// cancelation/confirmation
    limbo: bool,
}

impl AvatarTracker {
    fn new(turn: Turn) -> Self {
        Self { turn, limbo: false }
    }
}

/// State of a stage for a given turn.
#[derive(Clone)]
pub struct StageState {
    pub turn: StageTurn,
    pub foes: Vec<Foe>,
    pub orb: Orb,
    pub avatars: BTreeMap<AvatarId, Avatar>,
}

impl StageState {
    pub fn find_foe(&self, position: Position) -> Option<(usize, &Foe)> {
        self.foes
            .iter()
            .enumerate()
            .find(|(_, f)| f.position() == position)
    }
}

/// What's needed to recompute a stage state
#[derive(Clone, Default)]
struct StageDiff {
    diff_by_avatar: BTreeMap<AvatarId, StageAvatarDiff>,
    new_avatar: Option<Avatar>,
}

#[derive(Clone, Debug)]
struct StageAvatarDiff {
    action: ServerAction,
    senses: Senses,
}

pub struct StageCommandResult {
    pub stage_turn: StageTurn,
    pub limbos: Vec<Limbo>,
    pub senses_info: SensesInfo,
    pub action: ServerAction,
    pub logs: Vec<(StageTurn, GameLogEvent)>,
    pub transition: Option<Transition>,
}

fn orb_spawn(stage: &Stage, stage_turn: StageTurn) -> Position {
    let spawns: Vec<Position> = stage
        .template
        .orb_spawns
        .indexed_iter()
        .filter(|(_, val)| **val)
        .map(|(pos, _)| Position::from(pos))
        .collect();

    if spawns.is_empty() {
        warn!("Couldn't find a spawn point for lvl");
        return Default::default();
    }

    // Deterministic random selection based on seed and stage_turn
    // Using a simple hash combination
    let hash = stage
        .seed
        .wrapping_add(stage_turn)
        .wrapping_mul(6364136223846793005);
    let i = (hash as usize) % spawns.len();
    spawns[i]
}
