use serde::{Deserialize, Serialize};

use crate::{
    sense::SenseType,
    types::{FoeType, PlayerId},
};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct GEvent {
    /// Senses used to detect it
    sources: Vec<SenseType>,
    event: GameEvent,
}

impl GEvent {
    pub fn new(sources: Vec<SenseType>, event: GameEvent) -> Self {
        Self { sources, event }
    }

    pub fn sources(&self) -> &[SenseType] {
        &self.sources
    }

    pub fn event(&self) -> &GameEvent {
        &self.event
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum GameEvent {
    Attack {
        subject: Target,
        source: Target,
    },
    /// When a player tried to attack but rollback made it impossible to
    Fumble(Target),
    Kill {
        subject: Target,
        source: Target,
    },
    /// When a foe is killed by a death bound
    ParadoxDeath(FoeType),
    /// When a foe is teleported by a position bound
    ParadoxTeleport(FoeType),
    OrbSeen,
    OrbTaken(Target),
    AvatarFadedOut(Target),
}
impl GameEvent {
    pub fn has_player(&self, pid: PlayerId) -> bool {
        match self {
            GameEvent::Attack { subject, source } => {
                subject.is_player(pid) || source.is_player(pid)
            }
            GameEvent::Fumble(target) => target.is_player(pid),
            GameEvent::Kill { subject, source } => subject.is_player(pid) || source.is_player(pid),
            GameEvent::OrbTaken(target) => target.is_player(pid),
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum Target {
    Foe(FoeType),
    Avatar(PlayerId),
    You,
    Player(PlayerId, String),
    DiscardedAvatar,
    Unknown,
}
impl Target {
    fn is_player(&self, pid: PlayerId) -> bool {
        match self {
            Target::Avatar(id) => pid == *id,
            Target::You => true,
            Target::Player(id, _) => pid == *id,
            _ => false,
        }
    }
}
