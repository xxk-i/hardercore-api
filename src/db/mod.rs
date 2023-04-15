use std::{path::{PathBuf}, fs};

use actix_web::{Responder, HttpResponse};

use self::errors::*;

mod errors;

#[derive(Clone)]
pub struct Database {
    pub path: PathBuf,
    world_count: u64,
    current_world: u64,
}

impl Database {
    pub fn new(path: PathBuf) -> Result<Self, DatabaseError> {
        if !path.exists() {
            fs::create_dir(&path)?;
        }

        let mut db = Database { path: path, world_count: 0, current_world: 0 };

        db.initalize_db_directory()?;

        Ok(db)
    }

    pub fn from(path: PathBuf) -> Result<Self, DatabaseError> {
        let mut db = Database { path: path, world_count: 0, current_world: 0 };

        db.world_count = db.find_latest_world()?;
        db.switch_world(db.world_count);

        Ok(db)
    }

    fn initalize_db_directory(&mut self) -> Result<(), DatabaseError> {
        // error if our new directory is not empty
        if !self.path.read_dir()?.next().is_none() {
            return Err(DatabaseError)
        }

        fs::create_dir_all(self.path.clone().join("worlds/world1"))?;

        self.world_count = 1;
        self.switch_world(1).expect("Failed to switch to first world in new database");

        Ok(())
    }

    fn find_latest_world(&mut self) -> Result<u64, DatabaseError> {
        let mut highest = 0u64;

        for dir in self.path.join("worlds").read_dir()? {
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

        // self.world_count = highest;
        // self.switch_world(highest).expect("Failed to switch to latest found world");

        Ok(highest)
    }

    pub fn update_time_in_water(&mut self, uuid: String, time: u64) {
        unimplemented!()
    }

    pub fn world_death_event(&mut self) {
        self.switch_world(self.world_count);
    }

    pub fn create_world(&mut self) -> Result<(), DatabaseError> {
        self.world_count += 1;
        let string_numerical_suffix = self.world_count.to_string();
        let mut path = "worlds/world".to_owned();
        path.push_str(&string_numerical_suffix);

        fs::create_dir_all(self.path.join(path))?;

        self.switch_world(self.world_count).unwrap();

        Ok(())
    }

    pub fn get_path(&self) -> impl Responder {
        HttpResponse::Ok().body(self.path.to_str().unwrap().to_owned())
    }

    pub fn get_current_world(&self) -> impl Responder {
        HttpResponse::Ok().body(self.world_count.to_string())
    }

    pub fn switch_world(&mut self, world: u64) -> Result<(), WorldNotFoundError> {
        if world == 0 || world > self.world_count {
            return Err(errors::WorldNotFoundError {
                world_number: world
            })
        }

        self.current_world = world;
        Ok(())
    }

}