use std::{path::{PathBuf, self}, fs};

use actix_web::{Responder, HttpResponse};

use self::errors::*;

mod errors;

pub struct Database {
    pub path: PathBuf,
    world_count: u64,
    current_world: u64,
}

impl Database {
    pub fn new(path: PathBuf) -> Result<Self, DatabaseError> {
        let mut db_path = path.clone();
        db_path.push("db");

        if !db_path.exists() {
            fs::create_dir(db_path)?;
        }

        Ok(Database { path: path, world_count: 0, current_world: 0 })
    }

    pub fn update_time_in_water(&mut self, uuid: String, time: u64) {
        unimplemented!()
    }

    pub fn get_path(&self) -> impl Responder {
        HttpResponse::Ok().body(self.path.to_str().unwrap().to_owned())
    }

    pub fn get_current_world(&self) -> impl Responder {
        HttpResponse::Ok().body(self.world_count.to_string())
    }

    // pub fn switch_world(&mut self, world: u64) -> Result<(), WorldNotFoundError> {
    //     if world > self.world_count {
    //         return Err(errors::WorldNotFoundError {
    //             world_number: world
    //         })
    //     }

    //     self.current_world = world;
    //     Ok(())
    // }

    pub fn switch_world(&self, world: u64) -> Result<(), WorldNotFoundError> {
        if world > self.world_count {
            return Err(errors::WorldNotFoundError {
                world_number: world
            })
        }

        Ok(())
    }
}