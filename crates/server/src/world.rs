use std::collections::BTreeMap;

use anyhow::{Result, anyhow};
use grid::Grid;
use log::{error, info, warn};
use losig_core::{
    sense::{Senses, SensesInfo},
    types::{
        Avatar, AvatarId, ClientAction, Foe, GameLogEvent, GameOver, GameOverStatus, ServerAction,
        StageTurn, Tiles, Transition,
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
    MaybeDead(AvatarId),
    Averted(AvatarId, SensesInfo),
    TooFarBehind(Avatar),
}

/// Info returned by add_command. Game over data might concern other players as they can be saved
/// by another player.
pub struct CommandResult {
    pub stage: usize,
    pub stage_turn: StageTurn,
    pub limbos: Vec<Limbo>,
    pub senses_info: SensesInfo,
    pub action: ServerAction,
    pub logs: Vec<(StageTurn, GameLogEvent)>,
}

pub enum TransitionDestination {
    Stage(usize),
    End,
}

pub struct World {
    pub stage_id_by_avatar_id: BTreeMap<AvatarId, usize>,
    pub stages: Vec<Stage>,
    /// Third param of tuple is stageid
    pub morgue: BTreeMap<AvatarId, GameOver>,
}

impl World {
    pub fn new(stages: Vec<StageTemplate>) -> Self {
        World {
            stage_id_by_avatar_id: Default::default(),
            stages: stages.into_iter().map(Stage::new).collect(),
            morgue: Default::default(),
        }
    }

    pub fn create_avatar(&mut self, aid: AvatarId) {
        info!("Create a new avatar #{aid}");
        self.stage_id_by_avatar_id.insert(aid, 0);
        let stage = &mut self.stages[0];

        let avatar = Avatar::new(aid);
        stage.add_avatar(avatar);
    }

    pub fn retire_avatar(&mut self, aid: AvatarId) -> Option<GameOver> {
        if let Some(gameover) = self.morgue.remove(&aid) {
            return Some(gameover);
        }

        let stage_id = self.stage_id_by_avatar_id.get(&aid)?;
        let avatar = self.stages.get_mut(*stage_id)?.remove_avatar(aid)?;
        Some(GameOver::new(&avatar, GameOverStatus::Dead, *stage_id))
    }

    pub fn add_command(
        &mut self,
        aid: AvatarId,
        action: ClientAction,
        senses: Senses,
    ) -> Result<CommandResult> {
        let mut stage_id = *self
            .stage_id_by_avatar_id
            .get(&aid)
            .ok_or_else(|| anyhow!("No avatar #{aid} found."))?;

        let stage = &mut self.stages[stage_id];

        let stage_id = &mut stage_id;
        match stage.add_command(aid, action, senses.clone()) {
            Ok(StageCommandResult {
                mut stage_turn,
                mut limbos,
                mut senses_info,
                mut action,
                mut logs,
                transition,
            }) => {
                if let Some(transition) = transition {
                    match self.handle_transition(aid, transition, senses) {
                        Ok((tr_stage_id, scr)) => {
                            action = scr.action;
                            senses_info = scr.senses_info;
                            logs = scr.logs;
                            limbos.extend(scr.limbos);
                            *stage_id = tr_stage_id;
                            stage_turn = scr.stage_turn;
                        }
                        Err(e) => error!("Error while handling transition: {e}"),
                    }
                }

                self.handle_limbos(&limbos, *stage_id);
                let result = CommandResult {
                    stage: *stage_id,
                    stage_turn,
                    limbos,
                    senses_info,
                    action,
                    logs,
                };

                Ok(result)
            }
            Err(e) => {
                warn!("Error applying command: {e}");
                Err(e)
            }
        }
    }

    fn handle_limbos(&mut self, limbos: &[Limbo], stage_id: usize) {
        for status in limbos {
            if let Limbo::Dead(avatar) = status {
                self.morgue.insert(
                    avatar.id,
                    GameOver::new(avatar, GameOverStatus::Dead, stage_id),
                );
                self.stage_id_by_avatar_id.remove(&avatar.id);
            }
        }
    }

    fn find_destination(
        &self,
        transition: &Transition,
        previous_stage: usize,
    ) -> TransitionDestination {
        match transition {
            Transition::Orb => {
                let max_stage = self.stages.len() - 1;
                let next_stage = previous_stage + 1;
                if next_stage > max_stage {
                    TransitionDestination::End
                } else {
                    TransitionDestination::Stage(next_stage)
                }
            }
        }
    }

    fn find_stage_mut(&mut self, aid: AvatarId) -> Result<(usize, &mut Stage)> {
        let stage_id = *self
            .stage_id_by_avatar_id
            .get(&aid)
            .ok_or_else(|| anyhow!("No avatar {aid} found"))?;
        let stage = self
            .stages
            .get_mut(stage_id)
            .ok_or_else(|| anyhow!("No stage {stage_id} found."))?;
        Ok((stage_id, stage))
    }

    fn handle_transition(
        &mut self,
        aid: AvatarId,
        transition: Transition,
        senses: Senses,
    ) -> Result<(usize, StageCommandResult)> {
        let (stage_id, stage) = self.find_stage_mut(aid)?;
        let mut avatar = stage
            .remove_avatar(aid)
            .ok_or_else(|| anyhow!("Couldn't find avatar {aid} in stage for transition"))?;

        let limbos_from_leave = stage.handle_limbo();
        avatar.reset();

        match self.find_destination(&transition, stage_id) {
            TransitionDestination::End => {
                self.stage_id_by_avatar_id.remove(&aid);
                self.morgue
                    .insert(aid, GameOver::new(&avatar, GameOverStatus::Win, stage_id));
                Err(anyhow!("Nowhere to go when you're winning!"))
            }
            TransitionDestination::Stage(stage_id) => {
                self.stage_id_by_avatar_id.insert(aid, stage_id);
                let next_stage = &mut self.stages[stage_id];

                next_stage.add_avatar(avatar);
                let mut scr = next_stage.add_command(aid, ClientAction::Wait, senses)?;
                scr.limbos.extend(limbos_from_leave);
                Ok((stage_id, scr))
            }
        }
    }
}
