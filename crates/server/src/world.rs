use std::collections::BTreeMap;

use anyhow::{Result, anyhow};
use grid::Grid;
use log::{error, info, warn};
use losig_core::{
    sense::{Senses, SensesInfo},
    types::{
        Avatar, ClientAction, Foe, GameLogEvent, GameOver, GameOverStatus, PlayerId, ServerAction,
        StageId, StageTurn, Tiles, Timeline, Transition,
    },
};

use crate::stage::{Stage, StageCommandResult};

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

pub enum Limbo {
    Dead(Avatar),
    MaybeDead(PlayerId),
    Averted(PlayerId, SensesInfo),
    TooFarBehind(Avatar),
}

/// Info returned by add_command. Game over data might concern other players as they can be saved
/// by another player.
pub struct CommandResult {
    pub stage: StageId,
    pub stage_turn: StageTurn,
    pub limbos: Vec<Limbo>,
    pub senses_info: SensesInfo,
    pub action: ServerAction,
    pub logs: Vec<(StageTurn, GameLogEvent)>,
    pub timeline_updates: Vec<(StageId, Timeline)>,
}

pub enum TransitionDestination {
    Stage(StageId),
    End,
}

pub struct Player {
    pub id: PlayerId,
    pub stage: Option<StageId>,
    /// Copy of the last avatar sent to a stage
    pub last_avatar: Avatar,
    pub gameover: Option<GameOver>,
}

pub struct World {
    pub player_by_id: BTreeMap<PlayerId, Player>,
    pub stages: Vec<Stage>,
}

impl World {
    pub fn new(stages: Vec<StageTemplate>) -> Self {
        World {
            stages: stages.into_iter().map(Stage::new).collect(),
            player_by_id: Default::default(),
        }
    }

    pub fn new_player(&mut self, pid: PlayerId) {
        // Retire player if present
        self.retire_player(pid);

        info!("New player #{pid} created.");
        let new_player = Player {
            id: pid,
            stage: Some(0),
            last_avatar: Avatar::new(pid),
            gameover: None,
        };

        let stage = &mut self.stages[0];
        stage.add_player(&new_player);

        self.player_by_id.insert(pid, new_player);
    }

    pub fn retire_player(&mut self, pid: PlayerId) -> Option<GameOver> {
        let player = self.player_by_id.remove(&pid)?;

        if let Some(stage_id) = player.stage {
            let avatar = self.stages.get_mut(stage_id)?.remove_player(pid)?;
            Some(GameOver::new(&avatar, GameOverStatus::Dead, stage_id))
        } else {
            player.gameover
        }
    }

    pub fn add_command(
        &mut self,
        pid: PlayerId,
        action: ClientAction,
        senses: Senses,
    ) -> Result<CommandResult> {
        let player = self
            .player_by_id
            .get(&pid)
            .ok_or_else(|| anyhow!("No player #{pid} found."))?;
        let mut stage_id = player
            .stage
            .ok_or_else(|| anyhow!("Trying to transition when not in a stage"))?;
        let stage = self
            .stages
            .get_mut(stage_id)
            .ok_or_else(|| anyhow!("Stage not found"))?;

        let StageCommandResult {
            mut stage_turn,
            mut limbos,
            mut senses_info,
            mut action,
            mut logs,
            transition,
            timeline,
        } = stage.add_command(pid, action, senses.clone())?;

        let mut timeline_updates = vec![(stage_id, timeline)];
        if let Some(transition) = transition {
            match self.handle_transition(pid, transition, senses) {
                Ok((tr_stage_id, scr)) => {
                    action = scr.action;
                    senses_info = scr.senses_info;
                    logs = scr.logs;
                    stage_id = tr_stage_id;
                    stage_turn = scr.stage_turn;
                    limbos.extend(scr.limbos);
                    timeline_updates.push((tr_stage_id, scr.timeline));
                }
                Err(e) => error!("Error while handling transition: {e}"),
            }
        }
        self.handle_limbos(&limbos, stage_id);
        let result = CommandResult {
            stage: stage_id,
            stage_turn,
            limbos,
            senses_info,
            action,
            logs,
            timeline_updates,
        };

        Ok(result)
    }

    fn handle_limbos(&mut self, limbos: &[Limbo], stage_id: StageId) {
        for status in limbos {
            if let Limbo::Dead(avatar) = status {
                let player_id = avatar.player_id;
                let Some(player) = self.player_by_id.get_mut(&player_id) else {
                    warn!("Could not find player {player_id} for handling limbo");
                    continue;
                };

                player.gameover = Some(GameOver::new(avatar, GameOverStatus::Dead, stage_id));
                player.last_avatar = avatar.clone();
                player.stage = None;
            }
        }
    }

    fn handle_transition(
        &mut self,
        pid: PlayerId,
        transition: Transition,
        senses: Senses,
    ) -> Result<(StageId, StageCommandResult)> {
        let player = self
            .player_by_id
            .get_mut(&pid)
            .ok_or_else(|| anyhow!("Player not found."))?;

        let stage_id = player
            .stage
            .ok_or_else(|| anyhow!("Trying to transition when not in a stage"))?;
        let stage = self
            .stages
            .get_mut(stage_id)
            .ok_or_else(|| anyhow!("Stage not found"))?;

        let mut avatar = stage
            .remove_player(pid)
            .ok_or_else(|| anyhow!("Couldn't find avatar {pid} in stage for transition"))?;

        avatar.reset();
        player.last_avatar = avatar;

        let limbos_from_leave = stage.handle_limbo();

        match find_destination(&self.stages, &transition, stage_id) {
            TransitionDestination::End => {
                player.stage = None;
                player.gameover = Some(GameOver::new(
                    &player.last_avatar,
                    GameOverStatus::Win,
                    stage_id,
                ));
                // TODO: handle this more gracefully
                Err(anyhow!("Nowhere to go when you're winning!"))
            }
            TransitionDestination::Stage(stage_id) => {
                player.stage = Some(stage_id);
                let next_stage = &mut self.stages[stage_id];

                next_stage.add_player(player);
                let mut scr = next_stage.add_command(pid, ClientAction::Wait, senses)?;
                scr.limbos.extend(limbos_from_leave);
                Ok((stage_id, scr))
            }
        }
    }
    pub fn get_pids_for_stage(&self, stage: StageId) -> Vec<PlayerId> {
        self.stages
            .get(stage)
            .map(|st| st.get_pids())
            .unwrap_or_default()
    }
}

fn find_destination(
    stages: &[Stage],
    transition: &Transition,
    previous_stage: StageId,
) -> TransitionDestination {
    match transition {
        Transition::Orb => {
            let max_stage = stages.len() - 1;
            let next_stage = previous_stage + 1;
            if next_stage > max_stage {
                TransitionDestination::End
            } else {
                TransitionDestination::Stage(next_stage)
            }
        }
    }
}
