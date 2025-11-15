//! Tiled related code

use std::io::Cursor;

use anyhow::{Context, Result, anyhow};
use log::debug;
use losig_core::types::{Foe, Position, Tile};
use tiled::{Layer, Loader};

use crate::world::{Tiles, World};

struct AssetsReader {}

const TILESET: &[u8] = include_bytes!("../../../maps/tileset/editor.tsx");

const LVLS: &[&[u8]] = &[
    include_bytes!("../../../maps/lvl1.tmx"),
    include_bytes!("../../../maps/lvl2.tmx"),
    include_bytes!("../../../maps/lvl3.tmx"),
];

const FOE_ID: u32 = 1;
const SPAWN_ID: u32 = 2;
const WALL_ID: u32 = 4;

impl tiled::ResourceReader for AssetsReader {
    type Resource = Cursor<&'static [u8]>;
    type Error = std::io::Error;

    fn read_from(
        &mut self,
        path: &std::path::Path,
    ) -> std::result::Result<Self::Resource, Self::Error> {
        match path.to_str() {
            Some("lvl1") => Ok(Cursor::new(LVLS[0])),
            Some("lvl2") => Ok(Cursor::new(LVLS[1])),
            Some("lvl3") => Ok(Cursor::new(LVLS[2])),
            Some("tileset/editor.tsx") => Ok(Cursor::new(TILESET)),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Could not determine name from {path:?}"),
            )),
        }
    }
}

impl<'a> TryFrom<&tiled::TileLayer<'a>> for Tiles {
    type Error = anyhow::Error;

    fn try_from(value: &tiled::TileLayer<'a>) -> Result<Self, Self::Error> {
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
                    _ => Tile::Empty,
                };
                result.set(Position { x, y }, tile);
            }
        }
        Ok(result)
    }
}

impl TryFrom<&tiled::Map> for World {
    type Error = anyhow::Error;

    fn try_from(value: &tiled::Map) -> Result<Self, Self::Error> {
        println!("terrain");
        let terrain_layer = value
            .layers()
            .find(|l| l.name == "Terrain")
            .and_then(Layer::as_tile_layer)
            .ok_or(anyhow!("No terrain layer"))?;

        println!("foes");
        let foes_layer = value
            .layers()
            .find(|l| l.name == "Foes")
            .and_then(Layer::as_tile_layer)
            .ok_or(anyhow!("No foes layer"))?;

        Ok(World::new(
            Tiles::try_from(&terrain_layer)?,
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
    let lvl1 = loader
        .load_tmx_map("lvl1")
        .with_context(|| "Cannot load tmx")?;
    World::try_from(&lvl1)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn load_world_test() {
        assert!(load_world().is_ok());
    }
}

