use std::collections::BTreeMap;

use anyhow::{Result, anyhow};
use grid::Grid;
use log::{info, warn};
use losig_core::{
    events::GEvent,
    network::StageInfo,
    sense::{SenseType, Senses, SensesInfo},
    types::{
        Avatar, ClientAction, Foe, GameOver, GameOverStatus, PlayerId, ServerAction, StageId,
        StageTurn, Tiles, Timeline, TimelineType, Transition,
    },
};

use crate::stage::Stage;

/// Data of a stage that can not change with time or action players
#[derive(Debug, Clone)]
pub struct StageTemplate {
    pub id: String,
    pub name: String,
    pub tiles: Tiles,
    pub orb_spawns: Option<Grid<bool>>,
    pub foes: Vec<Foe>,
    pub fp_regen: u32,
    pub senses: Vec<SenseType>,
    pub timeline_length: u32,
    pub timeline_type: TimelineType,
}

impl StageTemplate {
    pub fn new(
        id: String,
        name: String,
        tiles: Tiles,
        orb_spawns: Option<Grid<bool>>,
        foes: Vec<Foe>,
        fp_regen: u32,
        senses: Vec<SenseType>,
        timeline_length: u32,
        timeline_type: TimelineType,
    ) -> Self {
        Self {
            id,
            name,
            tiles,
            foes,
            orb_spawns,
            fp_regen,
            senses,
            timeline_length,
            timeline_type,
        }
    }
}

impl From<&StageTemplate> for StageInfo {
    fn from(value: &StageTemplate) -> Self {
        StageInfo {
            name: value.name.clone(),
            timeline_length: value.timeline_length,
            senses: value.senses.clone(),
        }
    }
}

pub enum Limbo {
    Dead(PlayerId),
    MaybeDead(PlayerId),
    Averted(PlayerId, SensesInfo),
    TooFarBehind(PlayerId),
}

/// Info returned by add_command. Game over data might concern other players as they can be saved
/// by another player.
pub struct CommandResult {
    pub limbos: Vec<Limbo>,
    pub timeline_updates: Vec<(StageId, Timeline)>,
    pub outcome: CommandResultOutcome,
}

pub enum CommandResultOutcome {
    Turn {
        stage: StageId,
        stage_turn: StageTurn,
        info: Option<SensesInfo>,
        action: ServerAction,
        events: Vec<GEvent>,
        timeline: Timeline,
    },
    Transition {
        stage_id: StageId,
        stage_info: StageInfo,
        stage_turn: StageTurn,
        info: Option<SensesInfo>,
        timeline: Timeline,
    },
    Gameover(GameOver),
}

pub enum TransitionDestination {
    Stage(StageId),
    End,
}

pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub stage: Option<StageId>,
    /// Copy of the last avatar sent to a stage
    pub last_avatar: Avatar,
    pub gameover: Option<GameOver>,
}

pub struct World {
    pub player_by_id: BTreeMap<PlayerId, Player>,
    pub stages: Vec<Stage>,
    pub name_gen: usize,
}

impl World {
    pub fn new(stages: Vec<StageTemplate>) -> Self {
        World {
            stages: stages.into_iter().map(Stage::new).collect(),
            player_by_id: Default::default(),
            name_gen: 0,
        }
    }

    pub fn new_player(&mut self, pid: PlayerId, name: Option<String>) -> Result<CommandResult> {
        // Retire player if present
        self.retire_player(pid);

        let name = match name {
            Some(name) => name,
            None => {
                self.name_gen += 1;
                format!("P{}", self.name_gen)
            }
        };
        info!("New player #{pid} created.");
        let new_player = Player {
            id: pid,
            name,
            stage: Some(0),
            last_avatar: Avatar::new(pid),
            gameover: None,
        };

        let stage = &mut self.stages[0];
        let scr = stage.add_player(&new_player, Senses::default())?;

        self.player_by_id.insert(pid, new_player);

        Ok(CommandResult {
            limbos: scr.limbos,
            timeline_updates: vec![(0, scr.timeline)],
            outcome: CommandResultOutcome::Transition {
                stage_id: 0,
                stage_info: (&stage.template).into(),
                stage_turn: scr.stage_turn,
                info: scr.senses_info,
                timeline: scr.timeline,
            },
        })
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
        let stage_id = player
            .stage
            .ok_or_else(|| anyhow!("Trying to transition when not in a stage"))?;
        let stage = self
            .stages
            .get_mut(stage_id)
            .ok_or_else(|| anyhow!("Stage not found"))?;

        let scr = stage.add_command(pid, action, senses.clone())?;
        let timeline_updates = vec![(stage_id, scr.timeline)];

        let result = if let Some(transition) = &scr.transition {
            match self.handle_transition(pid, *transition, senses) {
                Ok(mut tr_scr) => {
                    tr_scr.limbos.extend(scr.limbos);
                    tr_scr.timeline_updates.extend(timeline_updates);
                    tr_scr
                }
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            CommandResult {
                limbos: scr.limbos,
                timeline_updates,
                outcome: CommandResultOutcome::Turn {
                    stage: stage_id,
                    stage_turn: scr.stage_turn,
                    info: scr.senses_info,
                    action: scr.action,
                    events: scr.events,
                    timeline: scr.timeline,
                },
            }
        };

        self.handle_limbos(&result.limbos, stage_id);

        let stage = &mut self.stages[stage_id];
        if stage.players.is_empty() {
            stage.reset();
        }
        Ok(result)
    }

    fn handle_limbos(&mut self, limbos: &[Limbo], stage_id: StageId) {
        for status in limbos {
            if let Limbo::Dead(player_id) = status {
                let Some(player) = self.player_by_id.get_mut(player_id) else {
                    warn!("Could not find player {player_id} for handling limbo");
                    continue;
                };

                player.gameover = Some(GameOver::new(
                    &player.last_avatar,
                    GameOverStatus::Dead,
                    stage_id,
                ));
                player.stage = None;
            }
        }
    }

    fn handle_transition(
        &mut self,
        pid: PlayerId,
        transition: Transition,
        senses: Senses,
    ) -> Result<CommandResult> {
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
                let gameover = GameOver::new(&player.last_avatar, GameOverStatus::Win, stage_id);
                player.gameover = Some(gameover.clone());
                Ok(CommandResult {
                    limbos: vec![],
                    timeline_updates: vec![],
                    outcome: CommandResultOutcome::Gameover(gameover),
                })
            }
            TransitionDestination::Stage(stage_id) => {
                player.stage = Some(stage_id);
                let next_stage = &mut self.stages[stage_id];

                let mut scr = next_stage.add_player(player, senses.clone())?;
                scr.limbos.extend(limbos_from_leave);

                Ok(CommandResult {
                    limbos: scr.limbos,
                    timeline_updates: vec![(stage_id, scr.timeline)],
                    outcome: CommandResultOutcome::Transition {
                        stage_id,
                        stage_info: (&next_stage.template).into(),
                        stage_turn: scr.stage_turn,
                        info: scr.senses_info,
                        timeline: scr.timeline,
                    },
                })
            }
        }
    }
    pub fn get_all_infos_for_stage(
        &self,
        stage: StageId,
    ) -> Vec<(PlayerId, StageTurn, SensesInfo)> {
        self.stages
            .get(stage)
            .map(|st| st.get_all_infos())
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
