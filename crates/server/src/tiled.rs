//! Tiled related code

use std::io::Cursor;
use std::str::FromStr;

use anyhow::{Result, anyhow};
use grid::Grid;
use losig_core::sense::SenseType;
use losig_core::types::{Foe, FoeType, Position, Tile, TimelineType, Tiles};
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
    "arena_corridor",
    "arena_big",
    "lvl_1",
    "lvl_2",
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

    // Read custom properties
    let name = value
        .properties
        .get("name")
        .and_then(|p| match p {
            tiled::PropertyValue::StringValue(s) => Some(s.as_str()),
            _ => None,
        })
        .unwrap_or(&id)
        .to_string();

    let fp_regen = value
        .properties
        .get("fp_regen")
        .and_then(|p| match p {
            tiled::PropertyValue::IntValue(v) => Some(*v as u32),
            _ => None,
        })
        .unwrap_or(4);

    let senses = value
        .properties
        .get("senses")
        .and_then(|p| match p {
            tiled::PropertyValue::StringValue(s) => Some(s.as_str()),
            _ => None,
        })
        .map(|s| {
            s.split(';')
                .filter_map(|sense| SenseType::from_str(sense.trim()).ok())
                .collect()
        })
        .unwrap_or_else(|| vec![
            SenseType::SelfSense,
            SenseType::Sight,
            SenseType::Touch,
            SenseType::Hearing,
        ]);

    let timeline_length = value
        .properties
        .get("timeline_length")
        .and_then(|p| match p {
            tiled::PropertyValue::IntValue(v) => Some(*v as u32),
            _ => None,
        })
        .unwrap_or(100);

    let timeline_type = value
        .properties
        .get("timeline_type")
        .and_then(|p| match p {
            tiled::PropertyValue::StringValue(s) => Some(s.as_str()),
            _ => None,
        })
        .and_then(|s| TimelineType::from_str(s).ok())
        .unwrap_or(TimelineType::Asynchronous);

    Ok(StageTemplate::new(
        id,
        name,
        convert_tiled(&terrain_layer)?,
        get_orb_spawns(&orb_layer)?,
        get_foes(&foes_layer)?,
        fp_regen,
        senses,
        timeline_length,
        timeline_type,
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

    let mut id = 0;
    for x in 0..width {
        for y in 0..height {
            let Some(tile) = layer.get_tile(x as i32, y as i32) else {
                continue;
            };
            let position = Position {
                x: x as usize,
                y: y as usize,
            };

            let foe = if tile.id() == MINDSNARE_ID {
                Foe {
                    id,
                    foe_type: FoeType::MindSnare,
                    position,
                    hp: 1,
                    attack: 3,
                }
            } else if tile.id() == SIMPLE_FOE_ID {
                Foe {
                    id,
                    foe_type: FoeType::Simple,
                    position,
                    hp: 3,
                    attack: 2,
                }
            } else {
                continue;
            };

            results.push(foe);
            id += 1;
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
    load_world(&["arena", "arena_corridor", "arena_big"])
}

#[allow(unused)]
pub fn load_remi() -> Result<World> {
    load_world(&["lvl_1", "lvl_2", "tuto_hearing", "tuto_sight", "arena_big"])
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

    #[test]
    fn load_properties_test() {
        use losig_core::sense::SenseType;
        use losig_core::types::TimelineType;

        let world = load_arena();
        assert!(world.is_ok());

        let world = world.unwrap();
        let template = &world.stages[0].template;

        assert_eq!(template.name, "Arena");
        assert_eq!(template.fp_regen, 4);
        assert_eq!(template.timeline_length, 100);
        assert_eq!(template.timeline_type, TimelineType::Asynchronous);
        assert_eq!(
            template.senses,
            vec![
                SenseType::SelfSense,
                SenseType::Sight,
                SenseType::Touch,
                SenseType::Hearing
            ]
        );
    }
}
