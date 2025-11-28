use losig_core::types::{Foe, FoeId, GameLogEvent, Position, Target};

use crate::world::{Stage, StageState};

pub fn act(foe: &Foe, _stage: &Stage, state: &mut StageState) -> Box<dyn FnOnce(&mut Foe)> {
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
            // Find the nearest avatar
            let avatar_opt = state
                .avatars
                .values_mut()
                .map(|avatar| (avatar.position.dist(pos), avatar))
                .filter(|(dist, _)| *dist < 5)
                .min_by_key(|(dist, _)| *dist);

            if let Some((dist, avatar)) = avatar_opt {
                if dist <= 1 {
                    // Attack: reduce avatar hp by 3
                    avatar.hp = avatar.hp.saturating_sub(3);
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
