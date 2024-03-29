use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::{self};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use actix_web::error;

use crate::info::Info;
use super::cache::ProfileCache;

// Error when attempting to switch to a world that doesn't/hasn't existed 
#[derive(Debug, Error)]
pub enum WorldError {
    #[error("World number {0} not found")]
    NotFound(u64),

    #[error("JSON deserialize failure from playerstats")]
    PlayerStatsJSONError(#[from] serde_json::Error),

    #[error("World IO Error")]
    IOError(#[from] std::io::Error),

    // #[error("Player not found")]
    // PlayerNotFound
}

impl error::ResponseError for WorldError {}

#[derive(Deserialize, Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct PlayerStats {
    pub display_name: String,
    pub skin_url: String,
    pub time_in_water: u64,
    pub time_in_nether: u64,
    pub damage_taken: u64,
    pub mobs_killed: u64,
    pub food_eaten: u64,
    pub experience_gained: u64
}

#[derive(Debug)]
pub struct World {
    pub player_stats: HashMap<String, PlayerStats>,
    pub uptime: u64,
    pub killer: Option<super::PlayerStats>,
    cache: ProfileCache,
}

impl World {
    pub fn new() -> World {
        World {
            player_stats: HashMap::new(),
            uptime: 0,
            killer: None,
            cache: ProfileCache::new()
        }
    }

    pub fn from(path: PathBuf) -> Result<World, WorldError> {
        let mut world = World::new();

        for file in path.read_dir()? {
            let file = file?;
            if file.file_type().unwrap().is_file() {
                if file.path().extension().unwrap().eq("json") {
                    if file.file_name().eq("global.json") {
                        let global_data: super::Global = serde_json::from_str(&fs::read_to_string(file.path())?)?;
                        world.uptime = global_data.uptime;
                    } else {
                        let data = fs::read_to_string(file.path())?;
                        let uuid = file.path().file_stem().unwrap().to_str().unwrap().to_owned();
                        let stats: super::PlayerStats = serde_json::from_str(&data)?;

                        world.player_stats.insert(uuid.clone(), stats);

                    }
                }
            }

        }

        Ok(world)
    }

    // adds a new PlayerStats to the world if it doesn't already exist
    pub async fn get_player_stats(&mut self, uuid: &String) -> &mut PlayerStats {
        let profile = self.cache.get(uuid).await;

        if !self.player_stats.contains_key(uuid) {
            let stats = PlayerStats {
                display_name: profile.name.clone(),
                skin_url: profile.get_skin_url(),
                ..Default::default()
            };

            self.player_stats.insert(uuid.clone(), stats);
        }

        self.player_stats.get_mut(uuid).unwrap()
    }

    pub async fn merge_stats(&mut self, uuid: String, info: Info) {
        let stats = self.get_player_stats(&uuid).await;

        if let Some(time) = &info.time_in_water {
            stats.time_in_water += time;
        }

        if let Some(time) = &info.time_in_nether {
            stats.time_in_nether += time;
        }

        if let Some(damage_taken)  = &info.damage_taken {
            stats.damage_taken += damage_taken;
        }

        if let Some(mobs_killed) = &info.mobs_killed {
            stats.mobs_killed += mobs_killed;
        }

        if let Some(food_eaten) = &info.food_eaten {
            stats.food_eaten += food_eaten;
        }

        if let Some(experienced_gained) = &info.experience_gained {
            stats.experience_gained += experienced_gained;
        }
    }
}