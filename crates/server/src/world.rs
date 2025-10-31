use std::{collections::HashMap, str::FromStr};

use losig_core::types::{Entity, EntityId, MAP_SIZE, Position, Tile};

#[derive(Debug, Clone)]
pub struct World {
    pub tick: u64,
    pub tiles: Tiles,
    pub entities: HashMap<EntityId, Entity>,
    /// retrieve the source, win the game.
    pub orb: Option<Position>,
    pub winner: Option<EntityId>,
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

    pub fn find_entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
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

pub fn load_world() -> World {
    let world_str = include_str!("../../../map.txt");
    let Ok(tiles) = Tiles::from_str(world_str) else {
        panic!()
    };

    let orb_pos = world_str
        .find("¤")
        .map(|i| Position::from_index(i, MAP_SIZE));

    World {
        tick: 0,
        tiles,
        orb: orb_pos,
        entities: HashMap::new(),
        winner: None,
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
