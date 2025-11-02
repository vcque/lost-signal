use losig_core::sense::{ProximitySense, SelfSense, TerrainSense, WorldSense};

/// Represents one of the senses of an avatar
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

impl Sense for Option<ProximitySense> {
    fn incr(&mut self) {
        match self {
            Some(w) => w.radius += 1,
            None => {
                self.replace(ProximitySense { radius: 1 });
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

impl Sense for Option<SelfSense> {
    fn decr(&mut self) {
        self.take();
    }

    fn incr(&mut self) {
        self.replace(SelfSense {});
    }
}
