use losig_core::{
    sense::{SenseInfo, TerrainInfo},
    types::{AvatarId, Offset, Position, Tile},
};

const VIEW_SIZE: usize = 256;
const START_POS: Position = Position {
    x: VIEW_SIZE / 2,
    y: VIEW_SIZE / 2,
};

#[derive(Debug, Clone)]
pub struct WorldView {
    pub id: AvatarId,
    pub tick: u64,
    pub tiles: [Tile; VIEW_SIZE * VIEW_SIZE],
    pub last_info: SenseInfo,
    pub viewer: Position,
    pub winner: Option<AvatarId>,
    pub broken: bool,
    pub signal: usize,
}

impl WorldView {
    pub fn new(id: AvatarId) -> WorldView {
        WorldView {
            id,
            tiles: [Tile::Unknown; VIEW_SIZE * VIEW_SIZE],
            tick: 0,
            last_info: SenseInfo::default(),
            viewer: START_POS,
            broken: false,
            winner: None,
            signal: 100,
        }
    }

    pub fn tile_from_viewer(&self, offset: Offset) -> Tile {
        if self.viewer.is_oob(VIEW_SIZE, VIEW_SIZE, offset) {
            Tile::Unknown
        } else {
            let pos = self.viewer + offset;
            self.tile_at(pos)
        }
    }

    pub fn tile_at(&self, pos: Position) -> Tile {
        let i = pos.x + VIEW_SIZE * pos.y;
        if i >= self.tiles.len() {
            return Tile::Unknown;
        }
        self.tiles[pos.x + VIEW_SIZE * pos.y]
    }

    pub fn update(&mut self, info: SenseInfo) {
        if let Some(ref terrain) = info.terrain {
            self.apply_terrain(terrain);
        }
        if let Some(ref selfs) = info.selfs {
            self.broken = selfs.broken;
            self.signal = selfs.signal;
        }

        self.last_info = info;
    }

    /// Add new info from the server
    pub fn apply_terrain(&mut self, terrain: &TerrainInfo) {
        let center = Position {
            x: terrain.radius,
            y: terrain.radius,
        };

        let iradius = terrain.radius as isize;
        let terrain_size = 2 * terrain.radius + 1;

        for x in (-iradius)..(iradius + 1) {
            for y in (-iradius)..(iradius + 1) {
                let offset = Offset { x, y };
                let info_pos = center + offset;
                let tile = terrain.tiles[info_pos.as_index(terrain_size)];
                if !matches!(tile, Tile::Unknown) {
                    let world_pos = self.viewer + offset;
                    self.tiles[world_pos.as_index(VIEW_SIZE)] = tile;
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

    /// Resets the world. Mostly after a clear or a win.
    pub fn clear(&mut self) {
        self.tiles.fill(Tile::Unknown);
        self.winner = None;
        self.viewer = START_POS;
        self.broken = false;
        self.signal = 100;
        self.last_info = SenseInfo::default();
    }
}
