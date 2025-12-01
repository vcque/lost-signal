use std::ops::{Add, Neg, Sub};

use grid::Grid;
use serde::{Deserialize, Serialize};

pub const HP_MAX: u8 = 10;
pub const FOCUS_MAX: u8 = 100;

/**
* Lists all possible commands that can be sent by a player to the game.
* A command is an input that (often) leads to a modification of the game state.
*/
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum ClientAction {
    Spawn,
    MoveOrAttack(Direction),
    Wait,
}

/**
* The actions the server has computed from the client request.
*/
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum ServerAction {
    Spawn,
    Wait,
    Move(Position),
    /// foe id, should stay server side though
    Attack(usize),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum Direction {
    Up,
    UpRight,
    UpLeft,
    Right,
    Left,
    DownRight,
    DownLeft,
    Down,
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Deserialize, Serialize, Copy)]
pub struct Offset {
    pub x: isize,
    pub y: isize,
}

impl Add<Offset> for Offset {
    type Output = Self;
    fn add(mut self, rhs: Offset) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self
    }
}

impl Sub<Offset> for Offset {
    type Output = Self;
    fn sub(mut self, rhs: Offset) -> Self::Output {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self
    }
}

impl Neg for Offset {
    type Output = Offset;

    fn neg(mut self) -> Self::Output {
        self.x = -self.x;
        self.y = -self.y;
        self
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl From<(usize, usize)> for Position {
    fn from((x, y): (usize, usize)) -> Self {
        Position { x, y }
    }
}

impl From<Position> for (usize, usize) {
    fn from(value: Position) -> Self {
        (value.x, value.y)
    }
}

impl Position {
    pub fn move_once(self, dir: Direction) -> Self {
        self + dir.offset()
    }

    pub fn from_index(index: usize, width: usize) -> Position {
        Position {
            x: index % width,
            y: index / width,
        }
    }

    pub fn as_offset(&self) -> Offset {
        Offset {
            x: self.x as isize,
            y: self.y as isize,
        }
    }

    /// Chebyshev distance
    pub fn dist(&self, other: &Self) -> usize {
        let self_dims = [self.x, self.y];
        let other_dims = [other.x, other.y];

        self_dims
            .into_iter()
            .zip(other_dims)
            .map(|(a, b)| a.abs_diff(b))
            .max()
            .unwrap()
    }

    /// Manhattan distance
    pub fn dist_manhattan(&self, other: &Self) -> usize {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}

impl Add<Offset> for Position {
    type Output = Position;

    fn add(self, offset: Offset) -> Self::Output {
        // I'm starting to consider 2D geometry crates
        Position {
            x: self.x.wrapping_add_signed(offset.x),
            y: self.y.wrapping_add_signed(offset.y),
        }
    }
}

impl Sub<Position> for Position {
    type Output = Offset;

    fn sub(self, rhs: Position) -> Self::Output {
        let x = self.x as isize - rhs.x as isize;
        let y = self.y as isize - rhs.y as isize;
        Offset { x, y }
    }
}

impl Direction {
    pub fn offset(&self) -> Offset {
        let (x, y) = match self {
            Direction::Up => (0, -1),
            Direction::UpRight => (1, -1),
            Direction::UpLeft => (-1, -1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
            Direction::DownRight => (1, 1),
            Direction::Down => (0, 1),
            Direction::DownLeft => (-1, 1),
        };
        Offset { x, y }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Tile {
    #[default]
    Unknown,
    Wall,
    Empty,
    Spawn,
    Pylon,
}

impl Tile {
    pub fn can_travel(&self) -> bool {
        !matches!(self, Self::Wall | Self::Pylon)
    }

    pub fn opaque(&self) -> bool {
        matches!(self, Self::Wall | Self::Pylon)
    }
}

pub type Turn = u64;
pub type StageTurn = u64;

pub type PlayerId = u32;
pub type StageId = usize;

#[derive(Clone)]
pub struct Avatar {
    pub player_id: PlayerId,
    pub position: Position,
    pub hp: u8,
    pub focus: u8,

    /// flag to represent an avatar which could not pay the cost of its senses
    pub tired: bool,
    pub turns: Turn,

    // TODO: should be elsewhere, will do for now
    /// If a player turn set this flag to true on his avatar, he is transitioned according to it.
    pub transition: Option<Transition>,

    pub logs: Vec<(StageTurn, GameLogEvent)>,
}

/// A transition is the move of an avatar from one stage to another.
/// It can occur only on a player move and no more than once per turn.
#[derive(Clone, Copy)]
pub enum Transition {
    /// Just go to the next stage
    Orb,
}

impl Avatar {
    pub fn new(player_id: PlayerId) -> Self {
        Avatar {
            player_id,
            position: Position { x: 1, y: 1 },
            hp: HP_MAX,
            focus: FOCUS_MAX,
            tired: false,
            turns: 1,
            transition: None,
            logs: vec![],
        }
    }

    pub fn is_dead(&self) -> bool {
        self.hp == 0
    }

    pub fn reset(&mut self) {
        *self = Avatar {
            turns: self.turns,
            ..Self::new(self.player_id)
        };
    }
}

#[derive(Debug, Clone)]
pub enum Foe {
    /// Some kind of trap
    MindSnare(Position),
    /// Classic mob with hp and attack on sight for testing purpose
    Simple(Position, u8),
}
impl Foe {
    pub fn position(&self) -> Position {
        match self {
            Self::MindSnare(pos) => *pos,
            Self::Simple(pos, _) => *pos,
        }
    }

    pub fn alive(&self) -> bool {
        match self {
            Self::MindSnare(_) => true,
            Self::Simple(_, hp) => *hp > 0,
        }
    }

    pub fn can_be_attacked(&self) -> bool {
        self.alive()
            && match self {
                Foe::MindSnare(_) => false,
                Foe::Simple(_, _) => true,
            }
    }

    pub fn foe_id(&self) -> FoeId {
        match self {
            Foe::Simple(_, _) => FoeId::Simple,
            Foe::MindSnare(_) => FoeId::MindSnare,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Orb {
    pub position: Position,
    /// If excited, it will change position next turn
    pub excited: bool,
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub struct Tiles {
    pub grid: Grid<Tile>,
}

impl Tiles {
    pub fn new(width: usize, height: usize) -> Self {
        Tiles {
            grid: Grid::new(width, height),
        }
    }

    pub fn width(&self) -> usize {
        self.grid.rows()
    }

    pub fn height(&self) -> usize {
        self.grid.cols()
    }

    pub fn center(&self) -> Position {
        Position {
            x: self.width() / 2,
            y: self.height() / 2,
        }
    }

    pub fn get(&self, index: impl TryInto<(usize, usize)>) -> Tile {
        let Ok((x, y)) = index.try_into() else {
            return Default::default();
        };
        self.grid.get(x, y).copied().unwrap_or_default()
    }

    pub fn at_offset_from_center(&self, offset: Offset) -> Tile {
        let position = self.center() + offset;
        self.get(position)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameOver {
    pub status: GameOverStatus,
    pub stage: u8,
    pub turns: Turn,
    pub score: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum GameOverStatus {
    Win,
    Dead,
}

impl GameOver {
    pub fn new(avatar: &Avatar, status: GameOverStatus, stage: usize) -> Self {
        let mut score: u64 = (stage as u64 + 1) * 100;
        score = score.saturating_sub(avatar.turns);
        score *= 100;
        if status == GameOverStatus::Win {
            score *= 2;
        }

        Self {
            status,
            stage: stage as u8,
            turns: avatar.turns,
            score,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum GameLogEvent {
    Attack { from: Target, to: Target },
    StageUp(Target),
    Defeated { from: Target, to: Target },
    Spawn,
    OrbSeen,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum Target {
    Foe(FoeId),
    You,
    OtherPlayer,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum FoeId {
    MindSnare,
    Simple,
}

/// Represents a timeline for a given stage
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug, Default)]
pub struct Timeline {
    pub head: StageTurn,
    pub tail: StageTurn,
}
