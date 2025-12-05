use losig_core::{events::GEvent, types::Turn};

#[derive(Default, Clone, Debug)]
pub struct GameLogs {
    inner: Vec<GameLog>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameLog {
    pub turn: Turn,
    pub log: LogEvent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LogEvent {
    Client(ClientLog),
    Server(GEvent),
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

    pub fn add(&mut self, turn: Turn, log: ClientLog) {
        let log = GameLog {
            turn,
            log: LogEvent::Client(log),
        };
        self.inner.push(log);
        self.inner.sort_by_key(|l| l.turn);
        self.inner.dedup();
    }

    pub fn add_server_events(&mut self, turn: Turn, events: Vec<GEvent>) {
        for event in events {
            self.inner.push(GameLog {
                turn,
                log: LogEvent::Server(event),
            });
        }
    }
}
