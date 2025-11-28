use std::collections::BTreeMap;

use grid::Grid;
use log::{info, warn};
use losig_core::{
    sense::{Senses, SensesInfo},
    types::{Avatar, AvatarId, ClientAction, Foe, GameLogEvent, GameOver, GameOverStatus, ServerAction, StageTurn, Tiles},
};

use crate::stage::Stage;

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
}

/// Info returned by add_command. Game over data might concern other players as they can be saved
/// by another player.
pub struct CommandResult {
    /// Stage turn
    pub stage_turn: StageTurn,
    pub stage_tail: StageTurn,
    pub limbos: Vec<Limbo>,
    pub senses_info: SensesInfo,
    pub action: ServerAction,
    pub logs: Vec<(StageTurn, GameLogEvent)>,
}

pub struct World {
    pub stage_id_by_avatar_id: BTreeMap<AvatarId, usize>,
    pub stages: Vec<Stage>,
    pub morgue: BTreeMap<AvatarId, Avatar>,
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

        let avatar = Avatar {
            id: aid,
            stage: 0,
            position: Default::default(),
            hp: 10,
            focus: 100,
            turns: 0,
            tired: false,
            logs: Default::default(),
        };

        stage.add_avatar(avatar);
    }

    pub fn retire_avatar(&mut self, aid: AvatarId) -> Option<GameOver> {
        if let Some(avatar) = self.morgue.remove(&aid) {
            return Some(GameOver::new(&avatar, GameOverStatus::Dead));
        }

        let stage_id = self.stage_id_by_avatar_id.get(&aid)?;
        let avatar = self.stages.get_mut(*stage_id)?.retire_avatar(aid)?;

        Some(GameOver::new(&avatar, GameOverStatus::Dead))
    }

    pub fn add_command(
        &mut self,
        aid: AvatarId,
        action: ClientAction,
        senses: Senses,
    ) -> Option<CommandResult> {
        let stage_id = *self.stage_id_by_avatar_id.get(&aid)?;
        let stage = &mut self.stages[stage_id];

        match stage.add_command(aid, action, senses) {
            Ok(result) => {
                for status in &result.limbos {
                    if let Limbo::Dead(avatar) = status {
                        self.morgue.insert(avatar.id, avatar.clone());
                        self.stage_id_by_avatar_id.remove(&avatar.id);
                    }
                }
                Some(result)
            }
            Err(e) => {
                warn!("Error applying command: {e}");
                None
            }
        }
    }
}
