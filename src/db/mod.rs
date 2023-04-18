use std::{path::{PathBuf}, fs::{self, File}, io::Write};
use actix_web::{Responder, HttpResponse};

mod errors;
use errors::*;

mod mojang;
mod cache;
mod world;
use world::*;

#[derive(Debug)]
pub struct Database {
    pub path: PathBuf,
    world: World,
    cache: cache::ProfileCache,
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
            cache: cache::ProfileCache::new(),
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
            cache: cache::ProfileCache::new(),
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

    fn find_latest_world(path: &PathBuf) -> Result<u64, DatabaseError> {
        let mut highest = 0u64;
        let path = path.join("worlds");

        let read_dir = match path.read_dir() {
            Ok(read_dir) => read_dir,
            Err(_) => return Err(DatabaseError::WorldsFolderNotFound(path)),
        };

        for dir in read_dir {
            let dir = dir?;
            if dir.file_type()?.is_dir() {
                let filename = dir.file_name();
                let split: Vec<&str> = filename.to_str().unwrap().split("world").collect();
                let last = split.last().unwrap();
                
                let world_number = last.parse::<u64>().unwrap();
                if world_number > highest {
                    highest = world_number;
                }
            }
        }

        Ok(highest)
    }

    fn initalize_db_directory(&mut self) -> Result<(), DatabaseError> {
        // error if our new directory is not empty
        if !self.path.read_dir()?.next().is_none() {
            return Err(DatabaseError::DatabaseNotEmpty)
        }

        fs::create_dir_all(self.path.clone().join("worlds/world1"))?;

        self.switch_world(1).expect("Failed to switch to first world in new database");

        Ok(())
    }

    pub fn save(&self) -> Result<(), DatabaseError> {
        for entry in &self.world.player_stats {
            let mut file = File::create(self.path.join(format!("worlds/world{}/{}.json", self.current_world, entry.0)))?;
            if serde_json::to_string(entry.1)?.as_bytes().is_empty() {
                println!("shits empty");
            }
            file.write_all(serde_json::to_string(entry.1)?.as_bytes())?;
        }

        Ok(())
    }

    pub async fn update_time_in_water(&mut self, uuid: &String, time: u64) -> Result<(), Box<dyn std::error::Error>> {
        self.world.try_add_new_player(uuid.clone(), self.cache.get(uuid).await.name.clone());

        self.world.player_stats.get_mut(uuid).unwrap().time_in_water += time;

        Ok(())
    }

    pub async fn update_damage_taken(&mut self, uuid: &String, damage: u64) -> Result<(), Box<dyn std::error::Error>> {
        self.world.try_add_new_player(uuid.clone(), self.cache.get(uuid).await.name.clone());
        self.world.player_stats.get_mut(uuid).unwrap().damage_taken += damage;

        Ok(())
    }

    pub async fn update_mobs_killed(&mut self, uuid: &String, count: u64) -> Result<(), Box<dyn std::error::Error>> {
        self.world.try_add_new_player(uuid.clone(), self.cache.get(uuid).await.name.clone());
        self.world.player_stats.get_mut(uuid).unwrap().mobs_killed += count;

        Ok(())
    }

    pub async fn update_food_eaten(&mut self, uuid: &String, count: u64) -> Result<(), Box<dyn std::error::Error>> {
        self.world.try_add_new_player(uuid.clone(), self.cache.get(uuid).await.name.clone());
        self.world.player_stats.get_mut(uuid).unwrap().food_eaten += count;

        Ok(())
    }

    pub async fn update_experience_gained(&mut self, uuid: &String, amount: u64) -> Result<(), Box<dyn std::error::Error>> {
        self.world.try_add_new_player(uuid.clone(), self.cache.get(uuid).await.name.clone());
        self.world.player_stats.get_mut(uuid).unwrap().experience_gained += amount;

        Ok(())
    }

    pub fn world_death_event(&mut self) {
        self.switch_world(self.world_count);
    }

    pub fn create_world(&mut self) -> Result<(), DatabaseError> {
        self.world_count += 1;

        fs::create_dir_all(self.path.join(format!("worlds/world{}", self.world_count)))?;

        self.switch_world(self.world_count).unwrap();

        Ok(())
    }

    pub fn get_path(&self) -> impl Responder {
        HttpResponse::Ok().body(self.path.to_str().unwrap().to_owned())
    }

    pub fn get_current_world(&self) -> impl Responder {
        HttpResponse::Ok().body(self.world_count.to_string())
    }

    pub fn switch_world(&mut self, world: u64) -> Result<(), Box<dyn std::error::Error>> {
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
}