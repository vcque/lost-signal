//! Tiled related code

use std::io::Cursor;

use anyhow::{Result, anyhow};
use log::debug;
use losig_core::types::{Foe, Position, Tile, Tiles};
use tiled::{Layer, Loader};

use crate::world::{Stage, World};

struct AssetsReader {}

const TILESET: &[u8] = include_bytes!("../../../maps/tileset/editor.tsx");

const STAGES: &[(&str, &[u8])] = &[
    ("lvl1", include_bytes!("../../../maps/lvl1.tmx")),
    ("lvl2", include_bytes!("../../../maps/lvl2.tmx")),
    ("lvl3", include_bytes!("../../../maps/lvl3.tmx")),
];

const FOE_ID: u32 = 1;
const SPAWN_ID: u32 = 2;
const WALL_ID: u32 = 4;
const PYLON_ID: u32 = 5;

impl tiled::ResourceReader for AssetsReader {
    type Resource = Cursor<&'static [u8]>;
    type Error = std::io::Error;

    fn read_from(
        &mut self,
        path: &std::path::Path,
    ) -> std::result::Result<Self::Resource, Self::Error> {
        match path.to_str() {
            Some("tileset/editor.tsx") => Ok(Cursor::new(TILESET)),
            Some(lvl) => {
                let bytes = STAGES
                    .iter()
                    .find_map(|(name, bytes)| if *name == lvl { Some(bytes) } else { None })
                    .unwrap();
                Ok(Cursor::new(bytes))
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Could not determine name from {path:?}"),
            )),
        }
    }
}

fn convert_tiled(value: &tiled::TileLayer) -> Result<Tiles> {
    let mut result = Tiles::empty(
        value.width().ok_or(anyhow!("width is needed"))? as usize,
        value.height().ok_or(anyhow!("height is needed"))? as usize,
    );

    debug!("width: {}, height: {}", result.width, result.height);
    for x in 0..result.width {
        for y in 0..result.height {
            let Some(tiled_tile) = value.get_tile(x as i32, y as i32) else {
                continue;
            };
            let tile = match tiled_tile.id() {
                SPAWN_ID => Tile::Spawn,
                WALL_ID => Tile::Wall,
                PYLON_ID => Tile::Pylon,
                _ => Tile::Empty,
            };
            result.set(Position { x, y }, tile);
        }
    }
    Ok(result)
}

impl TryFrom<&tiled::Map> for Stage {
    type Error = anyhow::Error;

    fn try_from(value: &tiled::Map) -> Result<Self, Self::Error> {
        let terrain_layer = value
            .layers()
            .find(|l| l.name == "Terrain")
            .and_then(Layer::as_tile_layer)
            .ok_or(anyhow!("No terrain layer"))?;
        let foes_layer = value
            .layers()
            .find(|l| l.name == "Foes")
            .and_then(Layer::as_tile_layer)
            .ok_or(anyhow!("No foes layer"))?;

        Ok(Stage::new(
            convert_tiled(&terrain_layer)?,
            get_foes(&foes_layer)?,
        ))
    }
}

fn get_foes(layer: &tiled::TileLayer) -> Result<Vec<Foe>> {
    let mut results = vec![];
    let width = layer.width().ok_or(anyhow!("no width"))?;
    let height = layer.height().ok_or(anyhow!("no height"))?;

    for x in 0..width {
        for y in 0..height {
            let Some(tile) = layer.get_tile(x as i32, y as i32) else {
                continue;
            };
            if tile.id() == FOE_ID {
                let position = Position {
                    x: x as usize,
                    y: y as usize,
                };
                results.push(Foe { position });
            }
        }
    }

    Ok(results)
}

pub fn load_world() -> Result<World> {
    let mut loader = Loader::with_reader(AssetsReader {});

    let stages = STAGES
        .iter()
        .map(|st| loader.load_tmx_map(st.0))
        .filter_map(|r| r.ok())
        .filter_map(|m| Stage::try_from(&m).ok())
        .collect();

    Ok(World::new(stages))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn load_world_test() {
        assert!(load_world().is_ok());
    }
}
