use std::str::FromStr;

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
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn move_once(self, dir: Direction) -> Self {
        match dir {
            Direction::Up => Position {
                x: self.x,
                y: self.y.saturating_sub(1),
            },
            Direction::Down => Position {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Left => Position {
                x: self.x.saturating_sub(1),
                y: self.y,
            },
            Direction::Right => Position {
                x: self.x + 1,
                y: self.y,
            },
            Direction::UpLeft => Position {
                x: self.x.saturating_sub(1),
                y: self.y.saturating_sub(1),
            },
            Direction::UpRight => Position {
                x: self.x + 1,
                y: self.y.saturating_sub(1),
            },
            Direction::DownLeft => Position {
                x: self.x.saturating_sub(1),
                y: self.y + 1,
            },
            Direction::DownRight => Position {
                x: self.x + 1,
                y: self.y + 1,
            },
        }
    }

    pub fn from(index: usize) -> Position {
        Position {
            x: index % MAP_SIZE,
            y: index / MAP_SIZE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Wall,
    Empty,
    Spawn,
    Orb,
}

impl FromStr for Tile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "#" => Ok(Tile::Wall),
            "." => Ok(Tile::Empty),
            "S" => Ok(Tile::Spawn),
            "O" => Ok(Tile::Orb),
            _ => Err(format!("Unknown tile character: '{}'", s)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Entity {
    pub id: u64,
    pub position: Position,
}
