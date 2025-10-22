use crate::world::Position;

#[derive(Debug, Clone, Copy)]
pub struct Entity {
    pub id: u64,
    pub position: Position,
}
