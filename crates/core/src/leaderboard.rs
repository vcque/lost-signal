use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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
        let mut result = Leaderboard::new();
        result.add(LeaderboardEntry::new("ponobo".to_owned(), 3, 1237));
        result.add(LeaderboardEntry::new("kiraill".to_owned(), 12, 4660));
        result.add(LeaderboardEntry::new("cabri".to_owned(), 5, 1801));
        result
    }
}

#[derive(Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub name: String,
    pub deaths: u32,
    pub turns: u32,
    pub score: u32,
}

impl LeaderboardEntry {
    pub fn new(mut name: String, deaths: u32, turns: u32) -> Self {
        name.truncate(8);
        LeaderboardEntry {
            name,
            deaths,
            turns,
            score: Self::score(deaths, turns),
        }
    }

    fn score(deaths: u32, turns: u32) -> u32 {
        let death_score = 100_u32.saturating_sub(deaths);
        let turn_score = 100_u32.saturating_sub(turns.isqrt());

        (death_score + turn_score) * 100
    }
}
