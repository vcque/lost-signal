#[derive(Default, Clone, Debug)]
pub struct GameLogs {
    inner: Vec<GameLog>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameLog {
    pub turn: u64,
    pub log: ClientLog,
}

/// Logs generated client-side
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientLog {
    Help,
    NextStage,
    Lost,
    Win,
}

impl GameLogs {
    pub fn logs(&self) -> &[GameLog] {
        &self.inner
    }

    pub fn add(&mut self, turn: u64, log: ClientLog) {
        let log = GameLog { turn, log };
        self.inner.push(log);
        self.inner.sort_by_key(|l| l.turn);
        self.inner.dedup();
    }
}
