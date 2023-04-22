use super::db::Database;
use serde::{Deserialize, Serialize};

pub struct APIData {
    pub database: Database,
    pub auth: String
}

#[derive(Deserialize)]
pub struct SwitchInfo {
    pub world: u64
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KillInfo {
    // UUID of player who died (killed the world)
    pub killer: String,

    // name of damage source that killed the player
    pub source_name: String,

    // name of damage source type applied that killed the player
    pub source_type: String
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")] // json naming convention
pub struct Info {
    pub auth: String,
    pub time_in_water: Option<u64>,
    pub damage_taken: Option<u64>,
    pub has_died: Option<bool>,
    pub mobs_killed: Option<u64>,
    pub food_eaten: Option<u64>,
    pub experience_gained: Option<u64>,
    pub kill_info: Option<KillInfo>
}