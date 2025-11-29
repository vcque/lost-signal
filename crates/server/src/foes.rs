use losig_core::types::{AvatarId, Foe, FoeId, GameLogEvent, Position, Target};

use crate::stage::{SenseBindings, Stage, StageState};

pub fn act(
    foe: &Foe,
    _stage: &Stage,
    state: &mut StageState,
    bindings: &SenseBindings,
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
                            from: Target::Foe(FoeId::MindSnare),
                        },
                    ));
                }
            }
        }

        Foe::Simple(pos, _) => {
            // Find a viable target
            let avatar_opt = target_selection(foe, state, bindings);

            if let Some(aid) = avatar_opt {
                let avatar = &mut state.avatars.get_mut(&aid).unwrap();
                let dist = avatar.position.dist(&foe.position());
                if dist <= 1 {
                    // Attack: reduce avatar hp by 3
                    avatar.hp = avatar.hp.saturating_sub(2);
                    avatar.logs.push((
                        state.turn,
                        GameLogEvent::Attack {
                            to: Target::You,
                            from: Target::Foe(FoeId::Simple),
                        },
                    ));
                } else {
                    // Move toward avatar
                    let offset = avatar.position - *pos;

                    let new_pos = Position {
                        x: pos.x.wrapping_add_signed(offset.x.signum()),
                        y: pos.y.wrapping_add_signed(offset.y.signum()),
                    };
                    return Box::new(move |f| {
                        if let Foe::Simple(pos, _) = f {
                            *pos = new_pos
                        }
                    });
                }
            }
        }
    }

    Box::new(|_| {})
}

fn target_selection<'a>(
    foe: &'a Foe,
    state: &'a StageState,
    bindings: &'a SenseBindings,
) -> Option<AvatarId> {
    for avatar in state.avatars.values() {
        if avatar.is_dead() {
            continue;
        }
        let aid = avatar.id;
        let bindings = bindings.avatars.get(&aid);

        let min_hp = bindings.map(|b| b.min_hp);
        if let Some(min_hp) = min_hp
            && avatar.hp < min_hp + 2 {
                continue;
            }

        let dist = foe.position().dist(&avatar.position);
        if dist > 4 {
            continue;
        }
        return Some(avatar.id);
    }

    None
}
