use log::{debug, info};
use losig_core::{
    sense::{SenseInfo, TerrainInfo, WorldInfo},
    types::{Offset, Position, Tile},
};

const VIEW_SIZE: usize = 128;

#[derive(Debug, Clone)]
pub struct WorldView {
    pub tick: u64,
    pub tiles: [Tile; VIEW_SIZE * VIEW_SIZE],
    pub last_info: SenseInfo,
}

impl WorldView {
    pub fn new() -> WorldView {
        WorldView {
            tiles: [Tile::Unknown; VIEW_SIZE * VIEW_SIZE],
            tick: 0,
            last_info: SenseInfo::default(),
        }
    }

    pub fn tile_from_center(&self, offset: Offset) -> Tile {
        let center = Position {
            x: VIEW_SIZE / 2,
            y: VIEW_SIZE / 2,
        };

        let pos = center + offset;
        self.tile_at(pos)
    }

    pub fn tile_at(&self, pos: Position) -> Tile {
        let i = pos.x + VIEW_SIZE * pos.y;
        if i >= self.tiles.len() {
            return Tile::Unknown;
        }
        self.tiles[pos.x + VIEW_SIZE * pos.y]
    }

    pub fn update(&mut self, info: SenseInfo) {
        debug!("update: {info:?}");
        if let Some(ref terrain) = info.terrain {
            self.apply_terrain(terrain);
        }
        if let Some(ref world) = info.world {
            self.apply_world(world);
        }

        self.last_info = info;
    }

    pub fn apply_world(&mut self, world: &WorldInfo) {
        self.tick = world.tick;
    }

    /// Add new info from the server
    pub fn apply_terrain(&mut self, terrain: &TerrainInfo) {
        // view is always centered
        let center = VIEW_SIZE / 2;
        let radius = terrain.radius;
        for x in 0..(2 * radius + 1) {
            for y in 0..(2 * radius + 1) {
                let tile = terrain.tiles[x + (2 * radius + 1) * y];
                if !matches!(tile, Tile::Unknown) {
                    info!("applying tile");
                    let x_view = center - radius + x;
                    let y_view = center - radius + y;
                    self.tiles[x_view + VIEW_SIZE * y_view] = tile;
                }
            }
        }
    }

    pub fn shift(&mut self, offset: Offset) {
        let mut new_tiles = [Tile::Unknown; VIEW_SIZE * VIEW_SIZE];

        for x in 0..VIEW_SIZE {
            for y in 0..VIEW_SIZE {
                let new_x = x as isize - offset.x;
                let new_y = y as isize - offset.y;

                if new_x >= 0
                    && new_x < VIEW_SIZE as isize
                    && new_y >= 0
                    && new_y < VIEW_SIZE as isize
                {
                    let old_idx = new_x as usize + VIEW_SIZE * new_y as usize;
                    let new_idx = x + VIEW_SIZE * y;
                    new_tiles[new_idx] = self.tiles[old_idx];
                }
            }
        }

        self.tiles = new_tiles;
    }
}
