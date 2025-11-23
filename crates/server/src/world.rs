use std::collections::HashMap;

use losig_core::types::{Avatar, AvatarId, Foe, Position, Tile, Tiles};

#[derive(Debug)]
pub struct World {
    pub tick: u64,
    pub avatars: HashMap<AvatarId, Avatar>,
    pub stages: Vec<Stage>,
}

impl World {
    pub fn new(stages: Vec<Stage>) -> World {
        World {
            tick: 0,
            stages,
            avatars: HashMap::new(),
        }
    }

    pub fn find_avatar(&self, id: AvatarId) -> Option<&Avatar> {
        self.avatars.get(&id)
    }
}

#[derive(Debug)]
pub struct Stage {
    pub tiles: Tiles,
    pub foes: Vec<Foe>,
    pub orb: Position,
}

impl Stage {
    pub fn new(tiles: Tiles, foes: Vec<Foe>) -> Self {
        let mut new = Self {
            tiles,
            foes,
            orb: Position::default(),
        };

        new.move_orb();
        new
    }

    pub fn find_spawns(&self) -> Vec<Position> {
        self.tiles
            .grid
            .indexed_iter()
            .filter_map(|((x, y), t)| {
                if *t == Tile::Spawn {
                    Some(Position { x, y })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn move_orb(&mut self) {
        loop {
            let x = rand::random_range(0..self.tiles.width());
            let y = rand::random_range(0..self.tiles.height());
            let position = Position { x, y };
            let tile = self.tiles.get(position);
            let foe = self.foes.iter().find(|f| f.position == position);

            if let (Tile::Empty, None) = (tile, foe) {
                self.orb = position;
                break;
            }
        }
    }
}
