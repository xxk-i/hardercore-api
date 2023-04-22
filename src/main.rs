use actix_web::{get, web::{self}, App, HttpResponse, HttpServer, Responder, put};
use serde::{Deserialize, Serialize};
use core::time;
use std::{sync::Mutex, fs};

mod db;

use db::Database;

pub struct APIData {
    pub database: Database,
    auth: String
}

#[derive(Deserialize)]
struct SwitchInfo {
    world: u64
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
    auth: String,
    pub time_in_water: Option<u64>,
    pub damage_taken: Option<u64>,
    pub has_died: Option<bool>,
    pub mobs_killed: Option<u64>,
    pub food_eaten: Option<u64>,
    pub experience_gained: Option<u64>,
    pub kill_info: Option<KillInfo>
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("You have reached the homepage of hardercore-api")
}

#[get("/world/current")]
pub async fn get_current_world(data: web::Data<Mutex<APIData>>) -> impl Responder {
    let db = &mut data.lock().unwrap().database;
    db.get_current_world()
}

#[get("database/path")]
pub async fn get_db_path(data: web::Data<Mutex<APIData>>) -> impl Responder {
    let db = &mut data.lock().unwrap().database;
    db.get_path()
}

#[put("/world/stats/{uuid}")]
async fn stats(data: web::Data<Mutex<APIData>>, info: web::Json<Info>, path: web::Path<(String,)>) -> impl Responder {
    let db = &mut data.lock().unwrap().database;

    let uuid = path.into_inner().0;

    match db.world.merge_stats(uuid, info.into_inner()) {
        Ok(_) => {},

        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };

    HttpResponse::Ok().body("OK")
}

#[put("/world")]
async fn switch_world(data: web::Data<Mutex<APIData>>, switch_info: web::Json<SwitchInfo>) -> impl Responder {
    let db = &mut data.lock().unwrap().database;

    return match db.switch_world(switch_info.world) {
        Ok(()) => {
            HttpResponse::Ok().body("OK")
        },

        Err(e) => {
            HttpResponse::BadRequest().body(e.to_string())
        }
    }
}

#[put("/world/create")]
async fn create_world(data: web::Data<Mutex<APIData>>) -> impl Responder {
    let db = &mut data.lock().unwrap().database;

    return match db.create_world() {
        Ok(()) => {
            HttpResponse::Ok().body("OK")
        },

        Err(e) => {
            HttpResponse::BadRequest().body(e.to_string())
        }
    }
}

#[get("/sleep")]
async fn sleep() -> impl Responder {
    std::thread::sleep(time::Duration::from_secs(5));
    HttpResponse::Ok()
}

#[get("/world/stats/{uuid}")]
async fn get_stats(data: web::Data<Mutex<APIData>>, path: web::Path<(String,)>) -> impl Responder {
    match data.lock().unwrap().database.get_player_stats(&path.into_inner().0) {
        Err(e) => {
            HttpResponse::BadRequest().body(e.to_string())
        },

        Ok(player_stats) => {
            HttpResponse::Ok().body(serde_json::to_string(player_stats).unwrap())
        },

    }
}

#[get("world/stats")]
async fn get_all_stats(data: web::Data<Mutex<APIData>>) -> impl Responder {
    let db = &data.lock().unwrap().database;

    HttpResponse::Ok().body(db.get_all_stats().expect("Saved player stats JSON is probably malformed!"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    color_eyre::install().unwrap();
    let path = std::env::current_dir().unwrap().join("db");
    let db = match path.exists() {
        true => {
            println!("Retrieving existing database");
            Database::from(path).expect("Something went wrong getting the existing database")
        },
        false => {
            println!("Creating new database");
            Database::new(path).expect("Something went wrong creating a new database")
        },
    };

    let api_data = web::Data::new(Mutex::new(APIData {
        database: db,
        auth: fs::read_to_string("auth.txt").expect("Auth setup failed")
    }));

    // syntax bullshit nightmare because async closures are unstable
    // so we make a normal closure that creates an environment with
    // a reference to api_data, then move that environment into our async block
    // allowing our loop to call the database to save every 2 minutes
    actix_rt::spawn((|api_data: web::Data<Mutex<APIData>>| {
        async move {
            let mut interval = actix_rt::time::interval(std::time::Duration::from_secs(10));
            loop {
                interval.tick().await;
                let unlocked = api_data.lock().unwrap();
                unlocked.database.save().expect("Database failed to save");
            }
        }
    })(api_data.clone()));

    // HttpServer server spawns "workers" equal to the number
    // of phyiscal cpu's / cores available in the system
    //
    // the db is created outside of HttpServer::new so it is only created once,
    // we clone this Data<T> wrapper to every app_data instance
    HttpServer::new(move || {
        App::new()
            .app_data(api_data.clone())
            .service(stats)
            .service(get_all_stats)
            .service(get_stats)
            .service(get_current_world)
            .service(get_db_path)
            .service(create_world)
            .service(switch_world)
    })
    .bind(("127.0.0.1", 8080))?
    .bind(("172.25.254.1", 8080))?
    .run()
    .await

}