//! Tiled related code

use std::io::Cursor;

use anyhow::{Result, anyhow};
use grid::Grid;
use losig_core::types::{Foe, Position, Tile, Tiles};
use tiled::{Layer, Loader};

use crate::world::{StageTemplate, World};

struct AssetsReader {}

const TILESET: &[u8] = include_bytes!("../../../maps/tileset/editor.tsx");

macro_rules! include_stages {
      ($($name:literal),* $(,)?) => {
          &[
              $(
                  ($name, include_bytes!(concat!("../../../maps/", $name, ".tmx"))),
              )*
          ]
      };
  }

const STAGES: &[(&str, &[u8])] = include_stages![
    "tuto_self",
    "tuto_touch",
    "tuto_hearing",
    "tuto_sight",
    "tuto_end",
    "arena",
    "arena_corridor"
];

const MINDSNARE_ID: u32 = 1;
const SIMPLE_FOE_ID: u32 = 6;
const SPAWN_ID: u32 = 2;
const ORB_ID: u32 = 3;
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
    let mut result = Tiles::new(
        value.width().ok_or(anyhow!("width is needed"))? as usize,
        value.height().ok_or(anyhow!("height is needed"))? as usize,
    );

    for x in 0..result.width() {
        for y in 0..result.height() {
            let Some(tiled_tile) = value.get_tile(x as i32, y as i32) else {
                continue;
            };
            let tile = match tiled_tile.id() {
                SPAWN_ID => Tile::Spawn,
                WALL_ID => Tile::Wall,
                PYLON_ID => Tile::Pylon,
                _ => Tile::Empty,
            };
            result.grid[(x, y)] = tile;
        }
    }
    Ok(result)
}

fn convert_map(id: String, value: &tiled::Map) -> Result<StageTemplate> {
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
    let orb_layer = value
        .layers()
        .find(|l| l.name == "Orb")
        .and_then(Layer::as_tile_layer)
        .ok_or(anyhow!("No Orb layer"))?;

    Ok(StageTemplate::new(
        id,
        convert_tiled(&terrain_layer)?,
        get_orb_spawns(&orb_layer)?,
        get_foes(&foes_layer)?,
    ))
}

fn get_orb_spawns(layer: &tiled::TileLayer) -> Result<Grid<bool>> {
    let width = layer.width().ok_or(anyhow!("no width"))? as usize;
    let height = layer.height().ok_or(anyhow!("no height"))? as usize;

    let mut grid = Grid::<bool>::new(width as usize, height as usize);
    for x in 0..width {
        for y in 0..height {
            let Some(tile) = layer.get_tile(x as i32, y as i32) else {
                continue;
            };
            if tile.id() == ORB_ID {
                grid[(x, y)] = true;
            }
        }
    }
    Ok(grid)
}

/// TODO: get foe templates instead of foes
fn get_foes(layer: &tiled::TileLayer) -> Result<Vec<Foe>> {
    let mut results = vec![];
    let width = layer.width().ok_or(anyhow!("no width"))?;
    let height = layer.height().ok_or(anyhow!("no height"))?;

    for x in 0..width {
        for y in 0..height {
            let Some(tile) = layer.get_tile(x as i32, y as i32) else {
                continue;
            };
            let position = Position {
                x: x as usize,
                y: y as usize,
            };
            if tile.id() == MINDSNARE_ID {
                results.push(Foe::MindSnare(position));
            } else if tile.id() == SIMPLE_FOE_ID {
                results.push(Foe::Simple(position, 4));
            }
        }
    }

    Ok(results)
}

#[allow(unused)]
pub fn load_tutorial() -> Result<World> {
    let tutos: Vec<&str> = STAGES
        .iter()
        .map(|stage| stage.0)
        .filter(|id| id.starts_with("tuto"))
        .collect();

    load_world(&tutos)
}

#[allow(unused)]
pub fn load_arena() -> Result<World> {
    load_world(&["arena_corridor"])
}

pub fn load_world(stage_ids: &[&str]) -> Result<World> {
    let mut stages = vec![];
    let mut loader = Loader::with_reader(AssetsReader {});

    for id in stage_ids {
        let map = loader.load_tmx_map(id)?;
        let stage = convert_map(id.to_string(), &map)?;
        stages.push(stage);
    }

    Ok(World::new(stages))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn load_world_test() {
        let world = load_arena();
        assert!(world.is_ok());

        let world = world.unwrap();
        assert!(world.stages.len() > 0);
    }
}
