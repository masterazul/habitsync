use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Habit {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub schedule: String,
    pub updated_at: u64,
    #[serde(default)]
    pub deleted: bool,
    #[serde(default)]
    pub seq: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Checkin {
    pub id: String,
    pub habit_id: String,
    pub date: String,
    pub updated_at: u64,
    #[serde(default)]
    pub deleted: bool,
    #[serde(default)]
    pub seq: u64,
}
