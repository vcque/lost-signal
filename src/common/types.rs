use std::{
    ops::{Add, Neg},
    str::FromStr,
};

use serde_derive::{Deserialize, Serialize};

pub const MAP_SIZE: usize = 256;

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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
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

impl Neg for Offset {
    type Output = Offset;

    fn neg(mut self) -> Self::Output {
        self.x = -self.x;
        self.y = -self.y;
        self
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
            Direction::DownLeft => (-1, 0),
        };
        Offset { x, y }
    }
}

impl Position {
    pub fn move_once(self, dir: Direction) -> Self {
        self + dir.offset()
    }

    pub fn from(index: usize) -> Position {
        Position {
            x: index % MAP_SIZE,
            y: index / MAP_SIZE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Wall,
    Empty,
    Spawn,
    Unknown,
}

impl FromStr for Tile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "#" => Ok(Tile::Wall),
            "¤" | "." => Ok(Tile::Empty),
            "S" => Ok(Tile::Spawn),
            " " => Ok(Tile::Unknown),
            _ => Err(format!("Unknown tile character: '{}'", s)),
        }
    }
}

pub type EntityId = u32;

#[derive(Debug, Clone, Copy)]
pub struct Entity {
    pub id: EntityId,
    pub position: Position,
}
