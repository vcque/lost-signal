use std::cmp::Ordering;

use losig_core::{
    network::GameLogsMessage,
    types::{GameLogEvent, StageTurn, Turn},
};

/// Number of turns to keep special log styling (averted and revision logs)
pub const LOG_RECENT_THRESHOLD: u64 = 5;

#[derive(Default, Clone, Debug)]
pub struct GameLogs {
    inner: Vec<GameLog>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameLog {
    pub turn: u64,
    pub log: LogEvent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LogEvent {
    Client(ClientLog),
    Server {
        stage_turn: StageTurn,
        received: Turn,

        /// If cancelled, the avatar turn at which it was averted
        averted: Option<Turn>,
        event: GameLogEvent,
    },
}

/// Logs generated client-side
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientLog {
    Help,
}

impl GameLogs {
    pub fn logs(&self) -> &[GameLog] {
        &self.inner
    }

    pub fn add(&mut self, turn: u64, log: ClientLog) {
        let log = GameLog {
            turn,
            log: LogEvent::Client(log),
        };
        self.inner.push(log);
        self.inner.sort_by_key(|l| l.turn);
        self.inner.dedup();
    }

    /// Assuming self and gamelogs vecs are ordered by turn incr and the ordering is stable
    pub fn merge(&mut self, server_logs: GameLogsMessage, current_turn: Turn, turn_diff: i64) {
        let mut results: Vec<GameLog> = vec![];

        let mut existing_iterator = self.inner.iter();
        let mut to_merge_iterator = server_logs.logs.iter();

        let make_existing = |existing: &GameLog| -> GameLog {
            let mut existing = existing.clone();
            if let LogEvent::Server { averted, .. } = &mut existing.log
                && averted.is_none()
            {
                *averted = Some(current_turn)
            }
            existing
        };

        let make_to_merge = |to_merge: &(StageTurn, GameLogEvent)| -> GameLog {
            let avatar_turn = to_merge.0.saturating_add_signed(turn_diff);
            GameLog {
                turn: avatar_turn,
                log: LogEvent::Server {
                    stage_turn: to_merge.0,
                    received: current_turn,
                    averted: None,
                    event: to_merge.1.clone(),
                },
            }
        };

        let mut next_existing = existing_iterator.next();
        let mut next_to_merge = to_merge_iterator.next();
        while next_existing.is_some() || next_to_merge.is_some() {
            match (next_existing, next_to_merge) {
                (Some(existing), Some(to_merge)) => match existing.log {
                    LogEvent::Client(_) => {
                        results.push(existing.clone());
                        next_existing = existing_iterator.next();
                    }
                    LogEvent::Server { ref event, .. } => {
                        let avatar_turn = to_merge.0.saturating_add_signed(turn_diff);
                        match avatar_turn.cmp(&existing.turn) {
                            Ordering::Less => {
                                results.push(make_to_merge(to_merge));
                                next_to_merge = to_merge_iterator.next();
                            }
                            Ordering::Greater => {
                                results.push(make_existing(existing));
                                next_existing = existing_iterator.next();
                            }
                            Ordering::Equal => {
                                if *event == to_merge.1 {
                                    results.push(existing.clone());
                                } else {
                                    results.push(make_existing(existing));
                                    results.push(make_to_merge(to_merge));
                                }

                                next_existing = existing_iterator.next();
                                next_to_merge = to_merge_iterator.next();
                            }
                        }
                    }
                },
                (Some(existing), None) => {
                    results.push(make_existing(existing));
                    next_existing = existing_iterator.next();
                }
                (None, Some(to_merge)) => {
                    results.push(make_to_merge(to_merge));
                    next_to_merge = to_merge_iterator.next();
                }
                (None, None) => unreachable!(),
            }
        }

        // Remove averted logs that are older than the recent threshold
        results.retain(|log| {
            if let LogEvent::Server {
                averted: Some(averted),
                ..
            } = &log.log
            {
                current_turn.saturating_sub(*averted) <= LOG_RECENT_THRESHOLD
            } else {
                true
            }
        });

        self.inner = results;
    }
}
