use std::{path::{PathBuf}, fs::{self, File}, io::Write};
use actix_web::{Responder, HttpResponse};

mod errors;
use errors::*;

mod mojang;
mod cache;
mod world;

use serde::Deserialize;
use serde::Serialize;
use world::*;

use super::util::KillInfo;

#[derive(Serialize, Deserialize)]
pub struct Global {
    pub uptime: u64
}

#[derive(Debug)]
pub struct Database {
    pub path: PathBuf,
    pub world: World,
    world_count: u64,
    current_world: u64
}

impl Database {
    pub fn new(path: PathBuf) -> Result<Self, DatabaseError> {
        if !path.exists() {
            fs::create_dir(&path)?;
        }

        let mut db = Database {
            path: path,
            world: World::new(),
            world_count: 0,
            current_world: 0
        };

        db.initalize_db_directory()?;

        Ok(db)
    }

    pub fn from(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let world_count = Database::count_worlds(&path)?;

        let mut db = Database {
            world: World::new(),
            path: path.clone(),
            world_count: world_count,
            current_world: 0
        };

        db.switch_world(world_count)?;

        Ok(db)
    }

    fn count_worlds(path: &PathBuf) -> Result<u64, DatabaseError> {
        let path = path.join("worlds");

        let read_dir = match path.read_dir() {
            Ok(read_dir) => read_dir,
            Err(_) => return Err(DatabaseError::WorldsFolderNotFound(path)),
        };

        let mut count = 0;
        for dir in read_dir {
            let dir = dir?;
            if dir.file_type()?.is_dir() {
                let filename = dir.file_name();
                if filename.to_str().unwrap().starts_with("world") {
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    fn initalize_db_directory(&mut self) -> Result<(), DatabaseError> {
        // error if our new directory is not empty
        if !self.path.read_dir()?.next().is_none() {
            return Err(DatabaseError::DatabaseNotEmpty)
        }

        self.create_world()?;
        self.switch_world(self.world_count).expect("Failed to switch to first world in new database");

        // fs::create_dir_all(self.path.clone().join("worlds/world1"))?;

        // self.switch_world(1).expect("Failed to switch to first world in new database");

        Ok(())
    }

    pub fn save(&self) -> Result<(), DatabaseError> {
        println!("Saving database...");

        // save all player stats to files labeled by uuid
        for entry in &self.world.player_stats {
            let mut file = File::create(self.path.join(format!("worlds/world{}/{}.json", self.current_world, entry.0)))?;

            // catch me never not pretty printing
            let pretty_string = serde_json::to_string_pretty(entry.1)?;

            file.write_all(pretty_string.as_bytes())?;
        }
        
        // save world uptime in global.json
        let mut file = File::create(self.path.join("global.json"));

        let mut global_data = Global {
            uptime: self.world.uptime
        };

        file.write_all(serde_json::to_string_pretty(&global_data)?.as_bytes()?)?;

        Ok(())
    }

    pub fn world_death_event(&mut self, kill_info: &KillInfo) -> Result<(), Box<dyn std::error::Error>> {
        println!("{} has killed the world", kill_info.killer);

        let mut file = File::create(self.path.join(format!("worlds/world{}/killed.json", self.current_world)))?;

        let kill_info_string = serde_json::to_string_pretty(kill_info)?;

        file.write_all(kill_info_string.as_bytes())?;

        // final save to all stats
        self.save()?;

        // clear the stats to prevent a weird save inbetween switching worlds
        self.world.player_stats.clear();

        self.create_world()?;
        self.switch_world(self.world_count)?;

        Ok(())
    }

    pub fn create_world(&mut self) -> Result<(), DatabaseError> {
        self.world_count += 1;

        fs::create_dir_all(self.path.join(format!("worlds/world{}", self.world_count)))?;

        Ok(())
    }

    pub fn get_path(&self) -> impl Responder {
        HttpResponse::Ok().body(self.path.to_str().unwrap().to_owned())
    }

    pub fn get_current_world(&self) -> impl Responder {
        HttpResponse::Ok().body(self.world_count.to_string())
    }

    pub fn switch_world(&mut self, world: u64) -> Result<(), Box<dyn std::error::Error>> {
        println!("Switching to world {}", world);

        if world == 0 || world > self.world_count {
            return Err(Box::new(WorldError::NotFound(world)))
        }

        self.world = World::from(self.path.join(format!("worlds/world{}", world)))?;

        self.current_world = world;
        Ok(())
    }

    pub fn get_player_stats(&self, uuid: &String) -> Result<&PlayerStats, DatabaseError> {
        match self.world.player_stats.get(uuid) {
            None => Err(DatabaseError::PlayerNotFound),
            Some(stats) => Ok(stats)
        }
    }

    pub fn get_all_stats(&self) -> Result<String, DatabaseError> {
        let stats: Vec<&PlayerStats> = self.world.player_stats.iter().map(|entry| entry.1).collect();

        Ok(serde_json::to_string_pretty(&stats)?)
    }
}