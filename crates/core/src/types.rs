use std::ops::{Add, Neg, Sub};

use grid::Grid;
use serde::{Deserialize, Serialize};

/**
* Lists all possible commands that can be sent by a player to the game.
* A command is an input that (often) leads to a modification of the game state.
*/
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum Action {
    Spawn,
    Move(Direction),
    Wait,
}

impl Action {
    pub fn allow_broken(&self) -> bool {
        matches!(self, Action::Spawn)
    }
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
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
pub type AvatarId = u32;

#[derive(Debug, Clone)]
pub struct Avatar {
    pub id: AvatarId,
    pub stage: usize,
    pub position: Position,
    pub hp: u8,
    pub focus: u8,

    /// flag to represent an avatar which could not pay the cost of its senses
    pub tired: bool,
    pub turns: Turn,
    /// This field is set when the player has won of lost
    pub gameover: Option<GameOver>,
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
    pub win: bool,
    pub stage: u8,
    pub turns: Turn,
    pub score: u64,
}

impl GameOver {
    pub fn new(avatar: &Avatar, win: bool) -> Self {
        let mut score: u64 = (avatar.stage as u64 + 1) * 100;
        score = score.saturating_sub(avatar.turns);
        score *= 100;
        if win {
            score *= 2;
        }

        Self {
            win,
            stage: (avatar.stage + 1) as u8,
            turns: avatar.turns,
            score,
        }
    }
}
