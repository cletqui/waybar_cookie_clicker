use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Clone)]
pub struct BuildingState {
    pub id: String,
    pub count: u32,
}

#[derive(Serialize, Deserialize, Default)]
pub struct GameState {
    pub cookies: f64,
    /// All-time cookies ever produced — never decreases, used for unlock conditions
    pub total_cookies: f64,
    pub last_tick_ms: u64,
    pub buildings: Vec<BuildingState>,
    pub upgrades_purchased: HashSet<String>,
}

impl GameState {
    pub fn building_count(&self, id: &str) -> u32 {
        self.buildings.iter().find(|b| b.id == id).map(|b| b.count).unwrap_or(0)
    }
}
