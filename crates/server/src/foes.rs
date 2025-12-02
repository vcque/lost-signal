use grid::Grid;
use losig_core::types::{Direction, Foe, FoeType, Position};

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
    let actions: Vec<FoeAction> = compute_possible_actions(foe, stage, state);
    if actions.len() == 1 {
        return actions[0];
    }

    let position_bound = compute_position_bounds(foe, stage, state, bindings);

    let actions = match &position_bound {
        None => actions,
        Some(bound) => filter_actions_by_bounds(foe, actions, bound),
    };

    let visible_avatars = find_visible_avatars(foe, state);

    select_best_action(&actions, &visible_avatars, foe, state, position_bound)
}

/// Compute all possible actions for a foe in the current situation
fn compute_possible_actions(foe: &Foe, stage: &Stage, state: &StageState) -> Vec<FoeAction> {
    let mut actions = vec![FoeAction::Wait];

    match foe.foe_type {
        FoeType::MindSnare => {
            // MindSnare can only attack avatars on its own tile
            if let Some((avatar_id, _)) = state
                .avatars
                .iter()
                .find(|(_, a)| a.position == foe.position)
            {
                actions.push(FoeAction::Attack(*avatar_id));
            }
        }
        FoeType::Simple => {
            // Simple foes can attack adjacent avatars and move normally
            const DIRECTIONS: [Direction; 8] = [
                Direction::Up,
                Direction::UpRight,
                Direction::Right,
                Direction::DownRight,
                Direction::Down,
                Direction::DownLeft,
                Direction::Left,
                Direction::UpLeft,
            ];

            for dir in DIRECTIONS {
                let new_pos = foe.position.move_once(dir);

                // Check if there's an avatar at this position
                if let Some((avatar_id, _)) =
                    state.avatars.iter().find(|(_, a)| a.position == new_pos)
                {
                    actions.push(FoeAction::Attack(*avatar_id));
                    continue;
                }

                // Check if the tile is walkable and not occupied by another foe
                let tile = stage.template.tiles.get(new_pos);
                if tile.can_travel() && state.find_foe(new_pos).is_none() {
                    actions.push(FoeAction::Move(new_pos));
                }
            }
        }
    }

    actions
}

/// Compute position bounds from the bindings as grids for evaluation
fn compute_position_bounds(
    foe: &Foe,
    stage: &Stage,
    state: &StageState,
    bindings: &SenseBounds,
) -> Option<Grid<u8>> {
    use std::collections::VecDeque;

    let width = stage.template.tiles.width();
    let height = stage.template.tiles.height();

    let bound = bindings
        .position_bounds
        .iter()
        .filter_map(|((foe_id, _), bound)| {
            if *foe_id == foe.id && bound.turn >= state.turn {
                Some(bound)
            } else {
                None
            }
        })
        .next()?;

    // Calculate turn difference once
    let turn_diff = (1 + bound.turn - state.turn) as u8;

    // Initialize result grid with zeros
    let mut result_grid: Grid<u8> = Grid::new(height, width);

    let Position { x, y } = bound.value;
    result_grid[(x, y)] = turn_diff;

    // BFS flood fill
    let mut queue = VecDeque::new();
    queue.push_back((bound.value, turn_diff));

    while let Some((pos, value)) = queue.pop_front() {
        if value == 0 {
            continue;
        }

        let next_value = value - 1;

        // Check all 8 directions
        const DIRECTIONS: [Direction; 8] = [
            Direction::Up,
            Direction::UpRight,
            Direction::Right,
            Direction::DownRight,
            Direction::Down,
            Direction::DownLeft,
            Direction::Left,
            Direction::UpLeft,
        ];

        for dir in DIRECTIONS {
            let next_pos = pos.move_once(dir);
            let Position { x: nx, y: ny } = next_pos;

            // Check bounds
            if nx >= width || ny >= height {
                continue;
            }

            // Check if tile is walkable
            let tile = stage.template.tiles.get(next_pos);
            if !tile.can_travel() {
                continue;
            }

            // Check if we can improve this cell
            let current = result_grid[(nx, ny)];
            if next_value > current {
                result_grid[(nx, ny)] = next_value;
                queue.push_back((next_pos, next_value));
            }
        }
    }

    Some(result_grid)
}

/// Filter actions respecting the bounds, ordered by the most respecting
fn filter_actions_by_bounds(
    foe: &Foe,
    actions: Vec<FoeAction>,
    position_bounds: &Grid<u8>,
) -> Vec<FoeAction> {
    let filtered = actions
        .iter()
        .filter(|a| {
            let Position { x, y } = a.next_position(foe);
            position_bounds.get(x, y).copied().unwrap_or_default() > 0
        })
        .copied()
        .collect::<Vec<_>>();

    if filtered.is_empty() {
        actions
    } else {
        filtered
    }
}

/// Find any avatar within a certain distance threshold
fn find_visible_avatars(foe: &Foe, state: &StageState) -> Vec<AvatarId> {
    state
        .avatars
        .iter()
        .filter(|(_, a)| a.position.dist(&foe.position) < 5)
        .map(|(id, _)| *id)
        .collect()
}

/// Select the best action, prioritizing: attacking > moving toward enemy > respecting position bounds
fn select_best_action(
    actions: &[FoeAction],
    visible_avatars: &[AvatarId],
    foe: &Foe,
    state: &StageState,
    position_bound: Option<Grid<u8>>,
) -> FoeAction {
    // Priority one: attack visible avatar
    for action in actions {
        if let FoeAction::Attack(target_id) = action
            && visible_avatars.contains(target_id)
        {
            return *action;
        }
    }

    // Priority two: move toward nearest visible avatar
    if !visible_avatars.is_empty() {
        let mut best_action: Option<(FoeAction, usize)> = None;

        for action in actions {
            let next_pos = action.next_position(foe);

            // Calculate minimum distance to any visible avatar from this position
            let target = visible_avatars
                .iter()
                .filter_map(|avatar_id| state.avatars.get(avatar_id))
                .min_by_key(|avatar| next_pos.dist(&avatar.position));

            if let Some(avatar) = target {
                let dist = avatar.position.dist_manhattan(&next_pos);
                match best_action {
                    None => best_action = Some((*action, dist)),
                    Some((_, best_dist)) if dist < best_dist => {
                        best_action = Some((*action, dist));
                    }
                    _ => {}
                }
            }
        }

        if let Some((action, _)) = best_action {
            return action;
        }
    }

    // Move toward position bound
    if let Some(bound) = position_bound {
        actions
            .iter()
            .copied()
            .max_by_key(|a| {
                let pos = a.next_position(foe);
                bound[pos.into()]
            })
            .unwrap_or_default();
    }

    FoeAction::Wait
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
enum FoeAction {
    #[default]
    Wait,
    Attack(AvatarId),
    Move(Position),
}

impl FoeAction {
    fn next_position(&self, foe: &Foe) -> Position {
        match self {
            FoeAction::Wait => foe.position,
            FoeAction::Attack(_) => foe.position,
            FoeAction::Move(position) => *position,
        }
    }
}

impl std::fmt::Display for FoeAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FoeAction::Wait => write!(f, "Wait"),
            FoeAction::Attack(id) => write!(f, "Attack({})", id),
            FoeAction::Move(pos) => write!(f, "Move({})", pos),
        }
    }
}
