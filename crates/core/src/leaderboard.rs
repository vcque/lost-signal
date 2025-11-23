use serde::{Deserialize, Serialize};

use crate::types::GameOver;

#[derive(Clone, Serialize, Deserialize)]
pub struct Leaderboard {
    entries: Vec<LeaderboardEntry>,
}

impl Leaderboard {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn add(&mut self, entry: LeaderboardEntry) {
        self.entries.push(entry);
        self.entries.sort_by_key(|e| e.gameover.score);
    }

    pub fn top_entries(&self, n: usize) -> &[LeaderboardEntry] {
        let end = n.min(self.entries.len());
        &self.entries[self.entries.len().saturating_sub(end)..]
    }
}

impl Default for Leaderboard {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub name: String,
    pub gameover: GameOver,
}

impl LeaderboardEntry {
    pub fn new(mut name: String, gameover: &GameOver) -> Self {
        name.truncate(8);
        LeaderboardEntry {
            name,
            gameover: gameover.clone(),
        }
    }
}
