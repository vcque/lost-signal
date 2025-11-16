use std::collections::HashMap;

use log::error;
use losig_core::types::{Avatar, AvatarId, Foe, Position, Tile};

#[derive(Debug)]
pub struct World {
    pub tick: u64,
    pub avatars: HashMap<AvatarId, Avatar>,
    pub stages: Vec<Stage>,
}

impl World {
    pub fn new(stages: Vec<Stage>) -> World {
        let new = World {
            tick: 0,
            stages,
            avatars: HashMap::new(),
        };

        new
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
            .buf
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                if *t == Tile::Spawn {
                    Some(Position::from_index(i, self.tiles.width))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn move_orb(&mut self) {
        loop {
            let x = rand::random_range(0..self.tiles.width);
            let y = rand::random_range(0..self.tiles.height);
            let position = Position { x, y };
            let tile = self.tiles.at(position);
            let foe = self.foes.iter().find(|f| f.position == position);

            match (tile, foe) {
                (Tile::Empty, None) => {
                    self.orb = position;
                    break;
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tiles {
    pub buf: Vec<Tile>,
    pub width: usize,
    pub height: usize,
}

impl Tiles {
    pub fn empty(width: usize, height: usize) -> Self {
        Tiles {
            buf: vec![Tile::Unknown; width * height],
            width,
            height,
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
