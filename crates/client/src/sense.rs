use losig_core::sense::{
    OrbSense, ProximitySense, SelfSense, SenseLevel, TerrainSense, WorldSense,
};

/// Represents one of the senses of an avatar
pub trait ClientSense {
    /// Make it stronger
    fn incr(&mut self);
    /// Make it weaker
    fn decr(&mut self);
}

impl ClientSense for Option<TerrainSense> {
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

impl ClientSense for Option<ProximitySense> {
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

impl ClientSense for Option<WorldSense> {
    fn decr(&mut self) {
        self.take();
    }

    fn incr(&mut self) {
        self.replace(WorldSense {});
    }
}

impl ClientSense for Option<SelfSense> {
    fn decr(&mut self) {
        self.take();
    }

    fn incr(&mut self) {
        self.replace(SelfSense {});
    }
}

impl ClientSense for Option<OrbSense> {
    fn incr(&mut self) {
        let level = match self {
            Some(w) => match w.level {
                SenseLevel::Minimum => SenseLevel::Low,
                SenseLevel::Low => SenseLevel::Medium,
                SenseLevel::Medium => SenseLevel::High,
                SenseLevel::High => SenseLevel::Maximum,
                SenseLevel::Maximum => SenseLevel::Maximum,
            },
            None => SenseLevel::Minimum,
        };

        self.replace(OrbSense { level });
    }

    fn decr(&mut self) {
        let level = match self {
            Some(w) => match w.level {
                SenseLevel::Minimum => {
                    self.take();
                    return;
                }
                SenseLevel::Low => SenseLevel::Minimum,
                SenseLevel::Medium => SenseLevel::Low,
                SenseLevel::High => SenseLevel::Medium,
                SenseLevel::Maximum => SenseLevel::High,
            },
            None => return,
        };

        self.replace(OrbSense { level });
    }
}
