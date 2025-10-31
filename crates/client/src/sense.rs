use losig_core::sense::{TerrainSense, WorldSense};

/// Represents one of the senses of an entity
pub trait Sense {
    /// Make it stronger
    fn incr(&mut self);
    /// Make it weaker
    fn decr(&mut self);
}

impl Sense for Option<TerrainSense> {
    fn incr(&mut self) {
        match self {
            Some(w) => w.radius += 1,
            None => {
                self.replace(TerrainSense { radius: 1 });
            }
        }
    }

    fn decr(&mut self) {
        match self {
            Some(w) => {
                if w.radius == 1 {
                    self.take();
                } else {
                    w.radius -= 1
                }
            }
            None => {}
        }
    }
}
impl Sense for Option<WorldSense> {
    fn decr(&mut self) {
        self.take();
    }

    fn incr(&mut self) {
        self.replace(WorldSense {});
    }
}
