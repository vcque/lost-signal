use losig_core::{
    events::{GEvent, GameEvent, Target},
    fov,
    sense::{SenseStrength, SenseType, Senses},
    types::{PlayerId, Position, Tile, Tiles},
};

use crate::stage::{Stage, StageState};

#[derive(Clone)]
pub struct GameEventSource {
    /// Senses that can detect it
    pub senses: EventSenses,
    /// If none, it means this is global
    pub source: EventSource,
    pub event: GameEvent,
}

#[derive(Clone)]
pub enum EventSource {
    Position(Position),
}

#[derive(Clone)]
pub enum EventSenses {
    All,
}
impl EventSenses {
    fn slice(&self) -> &[SenseType] {
        match self {
            EventSenses::All => &[
                SenseType::Touch,
                SenseType::Hearing,
                SenseType::Sight,
                SenseType::SelfSense,
            ],
        }
    }
}

pub fn gather_events(
    senses: &Senses,
    stage: &Stage,
    state: &StageState,
    pid: PlayerId,
) -> Vec<GEvent> {
    let avatar = &state.avatars[&pid];

    // Compute FOV once if sight is active
    let sight_tiles = if senses.sight.get() > 0 {
        Some(fov::fov(
            avatar.position,
            senses.sight.get().into(),
            &stage.template.tiles,
        ))
    } else {
        None
    };

    let mut detected_events = Vec::new();

    for event in state.events.get() {
        // The event is detected if:
        let mut detected_senses: Vec<SenseType> = vec![];
        for stype in event.senses.slice() {
            let detected = match stype {
                SenseType::SelfSense => senses.selfs.is_active() && event.event.has_player(pid),
                SenseType::Sight => {
                    senses.sight.is_active()
                        && (event.event.has_player(pid)
                            || is_seen(
                                avatar.position,
                                &event.source,
                                sight_tiles.as_ref().unwrap(),
                            ))
                }
                SenseType::Touch => {
                    senses.touch.is_active()
                        && (event.event.has_player(pid)
                            || is_touched(avatar.position, &event.source))
                }
                SenseType::Hearing => {
                    senses.hearing.is_active()
                        && (event.event.has_player(pid)
                            || is_heard(avatar.position, &event.source, senses.hearing.get()))
                }
            };

            if detected {
                detected_senses.push(*stype);
            }
        }

        if detected_senses.is_empty() {
            continue;
        }

        let transformed_event =
            transform_event_targets(&event.event, pid, stage, senses.sight.is_active());

        detected_events.push(GEvent::new(detected_senses, transformed_event));
    }

    detected_events
}

fn is_seen(viewer: Position, source: &EventSource, sight_tiles: &Tiles) -> bool {
    match source {
        EventSource::Position(pos) => {
            let offset = viewer - *pos;
            let view_pos = sight_tiles.center() + offset;
            sight_tiles.get(view_pos) != Tile::Unknown
        }
    }
}

fn is_touched(avatar_pos: Position, source: &EventSource) -> bool {
    match source {
        EventSource::Position(pos) => avatar_pos.dist(pos) <= 1,
    }
}

fn is_heard(avatar_pos: Position, source: &EventSource, hearing_strength: u8) -> bool {
    match source {
        EventSource::Position(pos) => avatar_pos.dist(pos) <= hearing_strength as usize,
    }
}

fn transform_event_targets(
    event: &GameEvent,
    pid: PlayerId,
    stage: &Stage,
    sight_active: bool,
) -> GameEvent {
    use GameEvent::*;

    let transform_target = |target: &Target| -> Target {
        match target {
            Target::Foe(foe_type) => {
                if sight_active {
                    Target::Foe(*foe_type)
                } else {
                    Target::Unknown
                }
            }
            Target::Avatar(id) => {
                if *id == pid {
                    Target::You
                } else if !sight_active {
                    Target::Unknown
                } else if let Some(player) = stage.players.get(id) {
                    Target::Player(*id, player.player_name.clone())
                } else {
                    Target::DiscardedAvatar
                }
            }
            Target::You | Target::Player(_, _) | Target::DiscardedAvatar | Target::Unknown => {
                target.clone()
            }
        }
    };

    match event {
        Attack { subject, source } => Attack {
            subject: transform_target(subject),
            source: transform_target(source),
        },
        Fumble(target) => Fumble(transform_target(target)),
        Kill { subject, source } => Kill {
            subject: transform_target(subject),
            source: transform_target(source),
        },
        ParadoxDeath(foe_type) => ParadoxDeath(*foe_type),
        ParadoxTeleport(foe_type) => ParadoxTeleport(*foe_type),
        OrbSeen => OrbSeen,
        OrbTaken(target) => OrbTaken(transform_target(target)),
        AvatarFadedOut(target) => AvatarFadedOut(transform_target(target)),
    }
}
