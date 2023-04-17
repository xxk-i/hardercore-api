use std::path::PathBuf;
use std::fs;

pub struct World {
    pub player_stats: Vec<super::PlayerStats>,
    pub uptime: u64,
    pub killer: Option<super::PlayerStats>
}

impl World {
    pub fn new() -> World {
        World {
            player_stats: Vec::new(),
            uptime: 0,
            killer: None
        }
    }

    pub fn from(path: PathBuf) -> World {
        for file in fs::read_dir(path).unwrap() {
            let file = file.unwrap();
            if file.file_type().unwrap().is_dir() {
            }
        }

        World::new()
    }
}