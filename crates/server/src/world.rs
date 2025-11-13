use std::{collections::HashMap, str::FromStr};

use log::error;
use losig_core::types::{Avatar, AvatarId, Foe, MAP_SIZE, Position, Tile};

#[derive(Debug, Clone)]
pub struct World {
    pub tick: u64,
    pub tiles: Tiles,
    pub avatars: HashMap<AvatarId, Avatar>,
    pub foes: Vec<Foe>,
    /// retrieve the orb win the game.
    pub orb: Option<Position>,
    pub winner: Option<AvatarId>,
}

impl World {
    pub fn find_free_spawns(&self) -> Vec<Position> {
        self.tiles
            .buf
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                if *t == Tile::Spawn {
                    Some(Position::from_index(i, MAP_SIZE))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn find_avatar(&self, id: AvatarId) -> Option<&Avatar> {
        self.avatars.get(&id)
    }
}

#[derive(Debug, Clone)]
pub struct Tiles {
    pub buf: Vec<Tile>,
    pub width: usize,
}

impl Tiles {
    pub fn empty(width: usize, height: usize) -> Self {
        Tiles {
            buf: vec![Tile::Unknown; width * height],
            width,
        }
    }

    pub fn at(&self, position: Position) -> Tile {
        let index = position.x + self.width * position.y;
        if index >= self.buf.len() {
            Tile::Unknown
        } else {
            self.buf[index]
        }
    }

    pub fn set(&mut self, position: Position, tile: Tile) {
        let index = position.x + self.width * position.y;
        if index < self.buf.len() {
            self.buf[index] = tile;
        } else {
            error!("Trying to set tiles oob: {:?}", position);
        }
    }
}

impl FromStr for Tiles {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut buf = vec![Tile::Unknown; MAP_SIZE * MAP_SIZE];

        for (y, row) in s.split("\n").enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let tile = Tile::from_str(&ch.to_string())?;
                buf[x + y * MAP_SIZE] = tile;
            }
        }

        Ok(Tiles {
            buf,
            width: MAP_SIZE,
        })
    }
}

pub fn load_world() -> World {
    let world_str = include_str!("../../../maps/simple.txt");
    let Ok(tiles) = Tiles::from_str(world_str) else {
        panic!()
    };

    let sp_tiles = find_special_tiles(world_str);

    let orb_pos = sp_tiles
        .iter()
        .filter(|(_, ch)| *ch == '¤')
        .map(|(p, _)| p)
        .next()
        .cloned();

    let foes: Vec<Foe> = sp_tiles
        .iter()
        .filter(|(_, ch)| *ch == 'µ')
        .map(|(p, _)| Foe { position: *p })
        .collect();

    World {
        tick: 0,
        tiles,
        orb: orb_pos,
        avatars: HashMap::new(),
        foes: foes,
        winner: None,
    }
}

pub fn find_special_tiles(world: &str) -> Vec<(Position, char)> {
    let mut results = vec![];
    for (y, row) in world.split("\n").enumerate() {
        for (x, ch) in row.chars().enumerate() {
            match ch {
                ' ' | '.' | 'S' => {}
                ch => {
                    let position = Position { x, y };
                    results.push((position, ch));
                }
            }
        }
    }

    results
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
