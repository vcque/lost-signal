use std::{
    ops::{Add, Neg, Sub},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

pub const MAP_SIZE: usize = 256;

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
        match self {
            Action::Spawn => true,
            _ => false,
        }
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
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

    pub fn as_index(&self, width: usize) -> usize {
        self.x + width * self.y
    }

    pub fn as_offset(&self) -> Offset {
        Offset {
            x: self.x as isize,
            y: self.y as isize,
        }
    }

    pub fn is_oob(&self, width: usize, height: usize, offset: Offset) -> bool {
        let ix = self.x as isize + offset.x;
        let iy = self.y as isize + offset.y;
        ix < 0 || iy < 0 || ix >= width as isize || iy >= height as isize
    }

    /// Chebyshev distance
    pub fn dist(&self, other: &Self) -> usize {
        let self_dims = [self.x, self.y];
        let other_dims = [other.x, other.y];

        self_dims
            .into_iter()
            .zip(other_dims.into_iter())
            .map(|(a, b)| a.abs_diff(b))
            .max()
            .unwrap()
    }
}

impl Add<Offset> for Position {
    type Output = Position;

    fn add(self, offset: Offset) -> Self::Output {
        Position {
            x: (self.x as isize).saturating_add(offset.x).max(0) as usize,
            y: (self.y as isize).saturating_add(offset.y).max(0) as usize,
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Tile {
    Wall,
    Empty,
    Spawn,
    Unknown,
}

impl Tile {
    pub fn can_travel(&self) -> bool {
        match self {
            Self::Wall | Self::Unknown => false,
            _ => true,
        }
    }

    pub fn opaque(&self) -> bool {
        match self {
            Self::Wall | Self::Unknown => true,
            _ => false,
        }
    }
}

impl FromStr for Tile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "#" => Ok(Tile::Wall),
            "S" => Ok(Tile::Spawn),
            " " => Ok(Tile::Unknown),
            _ => Ok(Tile::Empty), // Any special tile is an empty tile with an avatar/foe/anything on it
        }
    }
}

pub type AvatarId = u32;

#[derive(Debug, Clone)]
pub struct Avatar {
    pub id: AvatarId,
    pub position: Position,
    pub broken: bool,
    /// Some kind of energy, it's called signal because that's the name of the game
    pub signal: usize,
}

#[derive(Debug, Clone)]
pub struct Foe {
    pub position: Position,
}
