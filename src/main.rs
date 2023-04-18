use actix_web::{get, web::{self}, App, HttpResponse, HttpServer, Responder, put};
use serde::{Deserialize};
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct KillInfo {
    // UUID of player who died (killed the world)
    killer: String,

    // name of damage source that killed the player
    source_name: String,

    // name of damage source type applied that killed the player
    source_type: String
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")] // json naming convention
struct Info {
    auth: String,
    uuid: String,
    time_in_water: Option<u64>,
    damage_taken: Option<u64>,
    has_died: Option<bool>,
    mobs_killed: Option<u64>,
    food_eaten: Option<u64>,
    experience_gained: Option<u64>
}

#[derive(Deserialize)]
struct GetPlayerInfo {
    uuid: String
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

#[put("/world/stats")]
async fn stats(data: web::Data<Mutex<APIData>>, info: web::Json<Info>) -> impl Responder {
    let unlocked_data = &mut data.lock().unwrap();

    if !info.auth.eq(&unlocked_data.auth) {
        return HttpResponse::Forbidden()
    }

    let db = &mut unlocked_data.database;

    if let Some(time) = &info.time_in_water {
        db.update_time_in_water(&info.uuid, *time).await.unwrap();

        if cfg!(debug_assertions) {
            println!("{}'s time spent in water: {}", info.uuid, time);
        }
    }

    if let Some(damage_taken)  = &info.damage_taken {
        db.update_damage_taken(&info.uuid, *damage_taken).await.unwrap();
        println!("{}'s damage taken: {}", info.uuid, damage_taken);
    }

    if let Some(has_died)  = &info.has_died {
        println!("{} has died: {}", info.uuid, has_died);
    }

    if let Some(mobs_killed) = &info.mobs_killed {
        db.update_mobs_killed(&info.uuid, *mobs_killed).await.unwrap();
        println!("{} has killed: {}", info.uuid, mobs_killed);
    }

    if let Some(food_eaten) = &info.food_eaten {
        db.update_food_eaten(&info.uuid, *food_eaten).await.unwrap();
        println!("{} has eaten: {} food.", info.uuid, food_eaten);
    }

    if let Some(experienced_gained) = &info.experience_gained {
        db.update_experience_gained(&info.uuid, *experienced_gained).await.unwrap();
        println!("{} has gained: {} experience", info.uuid, experienced_gained);
    }

    HttpResponse::Ok()
}

#[put("/world/kill")]
async fn kill_world(data: web::Data<Mutex<APIData>>, ) -> impl Responder {
    HttpResponse::Ok()
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
            // .service(hello)
            .service(stats)
            .service(get_all_stats)
            .service(get_stats)
            .service(get_current_world)
            .service(get_db_path)
            .service(create_world)
            .service(switch_world)
            .service(kill_world)
            // .service(sleep)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await

}