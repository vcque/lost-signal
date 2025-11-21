use serde::{Deserialize, Serialize};

use crate::types::Turn;

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
        self.entries.sort_by_key(|e| e.score);
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
    pub deaths: u32,
    pub turns: Turn,
    pub score: u32,
}

impl LeaderboardEntry {
    pub fn new(mut name: String, deaths: u32, turns: Turn) -> Self {
        name.truncate(8);
        LeaderboardEntry {
            name,
            deaths,
            turns,
            score: Self::score(deaths, turns),
        }
    }

    fn score(deaths: u32, turns: Turn) -> u32 {
        let death_score = 100_u32.saturating_sub(deaths) * 100;
        let turn_score = 2000_u32.saturating_sub(turns as u32) * 5;

        death_score + turn_score
    }
}
