use std::collections::{BTreeMap, btree_map::Entry};

use losig_core::{
    sense::{SelfInfo, SightInfo},
    types::{Avatar, FoeId, PlayerId, Position, StageTurn},
};

/// When a player witness some states, it creates bindings on the previous states to ensure that
/// the witnessed state stays true

#[derive(Clone, Default)]
pub struct SenseBounds {
    pub avatars: BTreeMap<PlayerId, HpBound>,

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
            HpBound {
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

    pub fn release(&mut self, pid: PlayerId) {
        self.avatars.retain(|_, bound| bound.source != pid);
        self.death_bounds.retain(|_, bound| bound.source != pid);
        self.position_bounds.retain(|_, bound| bound.source != pid);
    }
}

/// The target health has been witnessed
#[derive(Clone)]
pub struct HpBound {
    pub value: u8,
    pub turn: StageTurn,
    pub source: PlayerId,
}

/// The target has been witnessed as dead at this turn.
#[derive(Clone)]
pub struct DeathBound {
    pub turn: StageTurn,
    pub source: PlayerId,
}

/// The target has been witnessed at a specific position.
#[derive(Clone)]
pub struct PositionBound {
    pub value: Position,
    pub turn: StageTurn,
    pub source: PlayerId,
}
