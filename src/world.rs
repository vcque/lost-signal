use std::{collections::HashMap, str::FromStr};

use serde_derive::{Deserialize, Serialize};

use crate::entity::Entity;

const MAP_SIZE: usize = 256;

#[derive(Debug, Clone)]
pub struct World {
    pub tick: u64,
    pub tiles: Tiles,
    pub entities: HashMap<u64, Entity>,
}

impl World {
    pub fn find_free_spawns(&self) -> Vec<Position> {
        self.tiles
            .buf
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                if *t == Tile::Spawn {
                    Some(Position::from(i))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tiles {
    buf: [Tile; MAP_SIZE * MAP_SIZE],
}

impl Tiles {
    pub fn at(&self, position: Position) -> Tile {
        self.buf[position.x + MAP_SIZE * position.y]
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

    fn from(index: usize) -> Position {
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

impl FromStr for Tiles {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut buf = [Tile::Empty; 256 * 256];

        for (i, ch) in s.chars().filter(|c| !c.is_whitespace()).enumerate() {
            if i >= 256 * 256 {
                return Err("Map data exceeds maximum size".to_string());
            }
            let tile = Tile::from_str(&ch.to_string())?;

            buf[i] = tile;
        }

        Ok(Tiles { buf })
    }
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

pub fn load_world() -> World {
    let world_str = include_str!("../map.txt");
    let Ok(tiles) = Tiles::from_str(world_str) else {
        panic!()
    };
    World {
        tick: 0,
        tiles,
        entities: HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_world() {
        let world = load_world();

        // Test that we can access the world structure
        assert!(matches!(world.tiles.buf[0], Tile::Wall)); // First char should be '#'

        // Find spawn position (should be 'S' in the map)
        let spawn_found = world
            .tiles
            .buf
            .iter()
            .any(|&tile| matches!(tile, Tile::Spawn));
        assert!(spawn_found, "Spawn tile should be present in the world");

        // Find orbs (should be 'O' in the map)
        let orb_found = world
            .tiles
            .buf
            .iter()
            .any(|&tile| matches!(tile, Tile::Orb));
        assert!(orb_found, "Orb tiles should be present in the world");

        // Check that we have walls
        let wall_found = world
            .tiles
            .buf
            .iter()
            .any(|&tile| matches!(tile, Tile::Wall));
        assert!(wall_found, "Wall tiles should be present in the world");

        // Check that we have empty spaces
        let empty_found = world
            .tiles
            .buf
            .iter()
            .any(|&tile| matches!(tile, Tile::Empty));
        assert!(empty_found, "Empty tiles should be present in the world");
    }
}
