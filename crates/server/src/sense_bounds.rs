use std::collections::{BTreeMap, btree_map::Entry};
use std::fmt;

use losig_core::events::GameEvent;
use losig_core::{
    sense::{SelfInfo, SightInfo},
    types::{Avatar, Foe, FoeId, PlayerId, Position, StageTurn},
};

use crate::events::{EventSenses, EventSource, GameEventSource};
use crate::stage::StageState;

/// When a player witness some states, it creates bindings on the previous states to ensure that
/// the witnessed state stays true

#[derive(Clone, Default, Debug)]
pub struct SenseBounds {
    pub avatars: BTreeMap<PlayerId, MaxHpBound>,

    pub death_bounds: BTreeMap<FoeId, DeathBound>,
    pub position_bounds: BTreeMap<(FoeId, PlayerId), PositionBound>,
}

impl SenseBounds {
    pub fn add_self_bounds(
        &mut self,
        pid: PlayerId,
        turn: StageTurn,
        avatar_id: PlayerId,
        selfi: &SelfInfo,
    ) {
        self.avatars.insert(
            avatar_id,
            MaxHpBound {
                value: selfi.hp,
                turn,
                source: pid,
            },
        );
    }

    pub fn add_sight_bounds(&mut self, avatar: &Avatar, turn: StageTurn, sight: &SightInfo) {
        let player_id = avatar.player_id;
        for foe in &sight.foes {
            if !foe.alive {
                match self.death_bounds.entry(foe.id) {
                    Entry::Vacant(vacant_entry) => {
                        vacant_entry.insert(DeathBound {
                            turn,
                            source: player_id,
                        });
                    }
                    Entry::Occupied(mut occupied_entry) => {
                        occupied_entry.get_mut().turn = occupied_entry.get().turn.min(turn);
                    }
                }
            } else {
                self.position_bounds.insert(
                    (foe.id, player_id),
                    PositionBound {
                        value: avatar.position + foe.offset,
                        turn,
                        source: player_id,
                    },
                );
            }
        }
    }

    pub fn enforce(&self, state: &mut StageState) {
        // Enforce avatar HP bounds
        for (avatar_id, hp_bound) in &self.avatars {
            if let Some(avatar) = state.avatars.get_mut(avatar_id) {
                hp_bound.enforce(state.turn, avatar);
            }
        }

        // Enforce foe death bounds
        for (foe_id, death_bound) in &self.death_bounds {
            if let Some(foe) = state.foes.get_mut(*foe_id)
                && death_bound.enforce(state.turn, foe) {
                    state.events.add(GameEventSource {
                        senses: EventSenses::All,
                        source: EventSource::Position(foe.position),
                        event: GameEvent::ParadoxDeath(foe.foe_type),
                    });
                }
        }

        // Enforce foe position bounds
        for ((foe_id, _player_id), position_bound) in &self.position_bounds {
            if let Some(foe) = state.foes.get_mut(*foe_id)
                && position_bound.enforce(state.turn, foe) {
                    state.events.add(GameEventSource {
                        senses: EventSenses::All,
                        source: EventSource::Position(foe.position),
                        event: GameEvent::ParadoxTeleport(foe.foe_type),
                    });
                }
        }
    }

    pub fn release(&mut self, pid: PlayerId) {
        self.avatars.retain(|_, bound| bound.source != pid);
        self.death_bounds.retain(|_, bound| bound.source != pid);
        self.position_bounds.retain(|_, bound| bound.source != pid);
    }
}

/// The target health has been witnessed
#[derive(Clone, Debug)]
pub struct MaxHpBound {
    pub value: u8,
    pub turn: StageTurn,
    pub source: PlayerId,
}

/// The target has been witnessed as dead at this turn.
#[derive(Clone, Debug)]
pub struct DeathBound {
    pub turn: StageTurn,
    pub source: PlayerId,
}

/// The target has been witnessed at a specific position.
#[derive(Debug, Clone)]
pub struct PositionBound {
    pub value: Position,
    pub turn: StageTurn,
    pub source: PlayerId,
}

impl PositionBound {
    pub fn enforce(&self, turn: StageTurn, foe: &mut Foe) -> bool {
        if turn == self.turn && foe.position != self.value {
            foe.position = self.value;
            true
        } else {
            false
        }
    }
}

impl fmt::Display for PositionBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PositionBound(pos: {}, turn: {}, source: {})",
            self.value, self.turn, self.source
        )
    }
}

impl DeathBound {
    pub fn enforce(&self, turn: StageTurn, foe: &mut Foe) -> bool {
        log::debug!("checking bounds {turn} -> {:?}", self);
        if turn == self.turn && foe.alive() {
            log::debug!("bound enforced");
            foe.hp = 0;
            true
        } else {
            false
        }
    }
}

impl MaxHpBound {
    pub fn enforce(&self, turn: StageTurn, avatar: &mut Avatar) {
        if turn == self.turn {
            avatar.hp = avatar.hp.max(self.value);
        }
    }
}
