use losig_core::types::{Offset, Position};

use crate::world::Tiles;

type F = fraction::Fraction;

enum Quadrant {
    East,
    West,
    North,
    South,
}

impl Quadrant {
    fn transform(&self, offset: &Offset) -> Offset {
        match self {
            Self::North => Offset {
                x: offset.x,
                y: -offset.y,
            },
            Self::East => Offset {
                x: offset.y,
                y: offset.x,
            },
            Self::West => Offset {
                x: -offset.y,
                y: -offset.x,
            },
            Self::South => *offset,
        }
    }
}

#[derive(Debug)]
struct Scanner {
    depth: usize,
    min_slope: F,
    max_slope: F,
}

impl Scanner {
    fn new() -> Self {
        Scanner {
            depth: 1,
            min_slope: F::from(-1),
            max_slope: F::from(1),
        }
    }

    fn offsets(&self) -> impl Iterator<Item = Offset> {
        let min_col: f32 = (self.min_slope * self.depth).try_into().unwrap();
        let max_col: f32 = (self.max_slope * self.depth).try_into().unwrap();

        let delta = 0.000001;
        let min_col = (min_col + delta).round() as isize;
        let max_col = (max_col - delta).round() as isize;

        (min_col..max_col + 1).map(|col| Offset {
            x: col,
            y: self.depth as isize,
        })
    }

    fn see_center(&self, offset: &Offset) -> bool {
        let low_bound = F::from(offset.x) >= self.min_slope * self.depth;
        let high_bound = F::from(offset.x) <= self.max_slope * self.depth;
        low_bound && high_bound
    }
}

/// This does a clone instead of a bit mask for now
pub fn fov(viewer: Position, radius: usize, tiles: &Tiles) -> Tiles {
    let mut result = Tiles::empty(2 * radius + 1, 2 * radius + 1);

    let center_view = Position {
        x: radius,
        y: radius,
    };

    for q in [
        Quadrant::East,
        Quadrant::West,
        Quadrant::North,
        Quadrant::South,
    ] {
        let mut scanners = vec![Scanner::new()];

        while let Some(scanner) = scanners.pop() {
            if scanner.depth > radius {
                continue;
            }

            let mut next_scan: Option<Scanner> = None;

            for offset in scanner.offsets() {
                let world_offset = q.transform(&offset);
                let position = viewer + world_offset;
                let tile = tiles.at(position);

                // 1. check if we show the tile
                if tile.opaque() || scanner.see_center(&offset) {
                    let result_pos = center_view + world_offset;
                    result.set(result_pos, tile);
                }
                // 2. Start a new scanner if we go from (nothing | opaque) -> see through
                if next_scan.is_none() && !tile.opaque() {
                    let slope = F::from(2 * offset.x - 1) / (2 * scanner.depth);
                    next_scan = Some(Scanner {
                        depth: scanner.depth + 1,
                        min_slope: slope,
                        max_slope: slope,
                    });
                }

                // 3. End a new scanner if we go from see through -> opaque
                if tile.opaque()
                    && let Some(mut ns) = next_scan
                {
                    let slope = F::from(2 * offset.x - 1) / (2 * scanner.depth);
                    ns.max_slope = slope;
                    scanners.push(ns);
                    next_scan = None;
                }
            }

            // If a scanner was started, end it with max-slope
            if let Some(mut ns) = next_scan {
                ns.max_slope = scanner.max_slope;
                scanners.push(ns);
            }
        }
    }

    result
}
