use losig_core::types::{Direction, Foe, FoeType, GameLogEvent, PlayerId, Position, Target};

use crate::{
    sense_bounds::SenseBounds,
    stage::{Stage, StageState},
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
    match foe {
        Foe::MindSnare(pos) => {
            for avatar in state.avatars.values_mut() {
                if *pos == avatar.position {
                    avatar.hp = avatar.hp.saturating_sub(3);
                    avatar.logs.push((
                        state.turn,
                        GameLogEvent::Attack {
                            to: Target::You,
                            from: Target::Foe(FoeType::MindSnare),
                        },
                    ));
                }
            }
        }

        Foe::Simple(pos, _) => {
            // Find a viable target
            let avatar_opt = target_selection(foe, state, bindings);

            if let Some((aid, hp_bound)) = avatar_opt {
                let avatar = &mut state.avatars.get_mut(&aid).unwrap();
                let dist = avatar.position.dist(&foe.position());
                if dist <= 1 {
                    if !hp_bound {
                        // Attack: reduce avatar hp by 3
                        avatar.hp = avatar.hp.saturating_sub(2);
                        avatar.logs.push((
                            state.turn,
                            GameLogEvent::Attack {
                                to: Target::You,
                                from: Target::Foe(FoeType::Simple),
                            },
                        ));
                    } else {
                        // Do nothing
                    }
                } else {
                    // Move toward avatar, avoiding other foes and avatars
                    let target_pos = avatar.position;

                    if let Some(new_pos) = find_best_move(*pos, target_pos, foe, stage, state) {
                        return Box::new(move |f| {
                            if let Foe::Simple(pos, _) = f {
                                *pos = new_pos
                            }
                        });
                    }
                }
            }
        }
    }

    Box::new(|_| {})
}

fn target_selection<'a>(
    foe: &'a Foe,
    state: &'a StageState,
    bindings: &'a SenseBounds,
) -> Option<(PlayerId, bool)> {
    let mut viable_targets: Vec<(PlayerId, bool, usize)> = vec![];

    for avatar in state.avatars.values() {
        if avatar.is_dead() {
            continue;
        }
        let aid = avatar.player_id;
        let bindings = bindings.avatars.get(&aid);

        let min_hp = bindings.map(|b| b.value);
        let is_hp_bound = if let Some(min_hp) = min_hp {
            avatar.hp < min_hp + 2
        } else {
            false
        };

        let dist = foe.position().dist(&avatar.position);
        if dist > 4 {
            continue;
        }
        viable_targets.push((aid, is_hp_bound, dist));
    }

    // Sort by (is_hp_bound, distance) - non-bound targets first, then by closest
    viable_targets.sort_by_key(|(_, is_bound, dist)| (*is_bound, *dist));

    viable_targets
        .first()
        .map(|(aid, hp_bound, _)| (*aid, *hp_bound))
}

/// Check if a position is occupied by a foe or avatar
fn is_position_occupied(pos: Position, current_foe: &Foe, state: &StageState) -> bool {
    // Check if any other foe occupies this position
    for foe in &state.foes {
        if foe.position() == pos && foe.position() != current_foe.position() && foe.alive() {
            return true;
        }
    }

    // Check if any avatar occupies this position
    for avatar in state.avatars.values() {
        if avatar.position == pos && !avatar.is_dead() {
            return true;
        }
    }

    false
}

/// Find the best move toward the target that avoids other foes, avatars, and walls
fn find_best_move(
    current_pos: Position,
    target_pos: Position,
    current_foe: &Foe,
    stage: &Stage,
    state: &StageState,
) -> Option<Position> {
    use Direction::*;

    // All possible directions
    let all_directions = [Up, UpRight, UpLeft, Right, Left, DownRight, DownLeft, Down];

    let current_dist = current_pos.dist(&target_pos);

    // Evaluate each direction and sort by distance improvement
    let mut candidates: Vec<(Direction, Position, usize)> = all_directions
        .iter()
        .map(|dir| {
            let new_pos = current_pos + dir.offset();
            let new_dist = new_pos.dist_manhattan(&target_pos);
            (*dir, new_pos, new_dist)
        })
        .collect();

    // Sort by distance (closest first)
    candidates.sort_by_key(|(_, _, dist)| *dist);

    // First pass: find traversable, unoccupied position that gets us closer or maintains distance
    for (_, new_pos, new_dist) in &candidates {
        let tile = stage
            .template
            .tiles
            .grid
            .get(new_pos.x, new_pos.y)
            .copied()
            .unwrap_or_default();
        if *new_dist <= current_dist
            && tile.can_travel()
            && !is_position_occupied(*new_pos, current_foe, state)
        {
            return Some(*new_pos);
        }
    }

    // All positions are blocked, don't move
    None
}
