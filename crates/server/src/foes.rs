use grid::Grid;
use losig_core::types::{Direction, Foe, FoeType, GameLogEvent, PlayerId, Position, Target};

use crate::{
    sense_bounds::SenseBounds,
    stage::{AvatarId, Stage, StageState},
};

pub fn act(
    foe: &Foe,
    stage: &Stage,
    state: &mut StageState,
    bindings: &SenseBounds,
) -> Box<dyn FnOnce(&mut Foe)> {
    if !foe.alive() {
        return Box::new(|_| {});
    }

    let action = foe_ai(foe, stage, state, bindings);

    match action {
        FoeAction::Attack(aid) => {
            if let Some(avatar) = state.avatars.get_mut(&aid) {
                avatar.hp = avatar.hp.saturating_sub(foe.attack);
            }
        }
        FoeAction::Wait => {}
        FoeAction::Move(position) => {
            return Box::new(move |f| f.position = position);
        }
    }

    Box::new(|_| {})
}

fn foe_ai(foe: &Foe, stage: &Stage, state: &mut StageState, bindings: &SenseBounds) -> FoeAction {
    // 1. List all possible actions
    let actions: Vec<FoeAction> = todo!("Compute all actions possible in this situation");
    if actions.is_empty() {
        return FoeAction::Wait;
    }

    let position_bounds: Vec<Grid<u8>> = todo!("Compute position bounds from the bindings");

    let in_bounds_actions: Vec<FoeAction> =
        todo!("Actions respecting the bounds, ordered by the most respecting");

    let visible_avatars = todo!("Any avatar with a dist < Val");

    todo!(
        "Best move selection, should prioritize attacking then going toward an enemy then toward the position bounds"
    );

    // Fallback to waiting
    FoeAction::Wait
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum FoeAction {
    Wait,
    Attack(AvatarId),
    Move(Position),
}
