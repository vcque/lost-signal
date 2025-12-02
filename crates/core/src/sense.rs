use bounded_integer::BoundedU8;
use serde::{Deserialize, Serialize};

use crate::types::{FoeId, FoeType, Offset, Tiles, Turn};

/// Describe information that an avatar want retrieved for a given turn
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct Senses {
    pub selfs: bool,
    pub touch: bool,
    pub sight: BoundedU8<0, 10>,
    pub hearing: BoundedU8<0, 5>,
}

impl Default for Senses {
    fn default() -> Self {
        Self {
            selfs: true,
            touch: false,
            sight: BoundedU8::const_new::<5>(),
            hearing: BoundedU8::const_new::<0>(),
        }
    }
}

impl Senses {
    pub fn cost(&self) -> u8 {
        let mut result = 0;
        if self.selfs {
            result += 1;
        }
        if self.touch {
            result += 1;
        }
        if self.sight > 0 {
            result += 2;
            result += self.sight;
        }
        result += self.hearing;

        result
    }

    pub fn merge(mut self, senses: Senses) -> Senses {
        self.touch = bool::merge(senses.touch, self.touch);
        self.selfs = bool::merge(senses.selfs, self.selfs);
        self.sight = BoundedU8::merge(senses.sight, self.sight);
        self.hearing = BoundedU8::merge(senses.hearing, self.hearing);
        self
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct SensesInfo {
    pub selfi: Option<SelfInfo>,
    pub touch: Option<TouchInfo>,
    pub sight: Option<SightInfo>,
    pub hearing: Option<HearingInfo>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct SelfInfo {
    pub hp: u8,
    pub hp_max: u8,
    pub focus: u8,
    pub turn: Turn,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct TouchInfo {
    pub tiles: Tiles,
    pub foes: u8,
    pub orb: bool,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct SightedFoe {
    pub id: FoeId,
    pub offset: Offset,
    pub foe_type: FoeType,
    pub alive: bool,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct SightedAlly {
    pub name: Option<String>,
    pub offset: Offset,
    pub alive: bool,
    pub status: SightedAllyStatus,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum SightedAllyStatus {
    Trailing,
    /// Can contain the move offset
    Leading(Option<Offset>),
    Sync,
    /// when the player has left the stage
    Abandonned,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct SightInfo {
    pub tiles: Tiles,
    pub foes: Vec<SightedFoe>,
    pub orb: Option<Offset>,
    pub allies: Vec<SightedAlly>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct HearingInfo {
    /// We don't want to give the player the exact distance
    pub range: Option<BoundedU8<1, 5>>,
}

impl HearingInfo {
    pub fn dist(strength: u8) -> Option<u8> {
        match strength {
            1 => Some(3),
            2 => Some(6),
            3 => Some(10),
            4 => Some(15),
            5 => Some(21),
            _ => None,
        }
    }
}

pub trait SenseStrength: Eq + Sized {
    fn max() -> Self;
    fn min() -> Self;
    fn decr(self) -> Self;
    fn incr(self) -> Self;
    fn merge(left: Self, right: Self) -> Self;

    fn is_min(&self) -> bool {
        *self == Self::min()
    }
}

impl SenseStrength for bool {
    fn max() -> Self {
        true
    }

    fn min() -> Self {
        false
    }

    fn incr(self) -> Self {
        true
    }

    fn decr(self) -> Self {
        false
    }

    fn merge(left: Self, right: Self) -> Self {
        left | right
    }
}

impl<const MIN: u8, const MAX: u8> SenseStrength for BoundedU8<MIN, MAX> {
    fn max() -> Self {
        Self::const_new::<MAX>()
    }

    fn min() -> Self {
        Self::const_new::<MIN>()
    }

    fn incr(self) -> Self {
        self.saturating_add(1)
    }

    fn decr(self) -> Self {
        self.saturating_sub(1)
    }

    fn merge(left: Self, right: Self) -> Self {
        left.max(right)
    }
}
