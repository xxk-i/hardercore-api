use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, put};
use serde::{Deserialize, de::Expected};
use std::{sync::Mutex, fs};

mod db;
use db::Database;

pub struct APIData<'a> {
    pub database: Mutex<&'a mut Database>,
    auth: String,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct SwitchInfo {
    world: u64
}

#[derive(Deserialize)]
#[allow(non_snake_case)] // we use json naming convention here
struct Info {
    auth: String,
    UUID: String,
    timeInWater: Option<u64>,
    damageTaken: Option<u64>,
    hasDied: Option<bool>,
    mobsKilled: Option<u64>,
    foodEaten: Option<u64>,
    experienceGained: Option<u64>
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("You have reached the homepage of hardercore-api")
}

#[get("/world/current")]
pub async fn get_current_world(data: web::Data<APIData>) -> impl Responder {
    // HTTPRequest::app_data().database.get_current_world()
    let db = data.database.lock().unwrap();
    db.get_current_world()
}

#[get("database/path")]
pub async fn get_db_path(data: web::Data<APIData>) -> impl Responder {
    let db = data.database.lock().unwrap();
    db.get_path()
}

#[put("/world/stats")]
async fn stats(data: web::Data<APIData>, info: web::Json<Info>) -> impl Responder {
    let _db = data.database.lock().unwrap();

    if !info.auth.eq(&data.auth) {
        return HttpResponse::Forbidden()
    }

    if let Some(time) = &info.timeInWater {
        println!("{}'s time spent in water: {}", info.UUID, time);
    }

    if let Some(damage_taken)  = &info.damageTaken {
        println!("{}'s damage taken: {}", info.UUID, damage_taken);
    }

    if let Some(has_died)  = &info.hasDied {
        println!("{} has died: {}", info.UUID, has_died);
    }

    if let Some(mobs_killed) = &info.mobsKilled {
        println!("{} has killed: {}", info.UUID, mobs_killed);
    }

    if let Some(food_eaten) = &info.foodEaten {
        println!("{} has eaten: {} food.", info.UUID, food_eaten);
    }

    if let Some(experienced_gained) = &info.experienceGained {
        println!("{} has gained: {} experience", info.UUID, experienced_gained);
    }

    HttpResponse::Ok()
}

#[put("/world")]
async fn switch_world(data: web::Data<APIData>, switch_info: web::Json<SwitchInfo>) -> impl Responder {
    let mut db = data.database.lock().unwrap();

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
async fn create_world(data: web::Data<APIData>) -> impl Responder {
    let mut db = data.database.lock().unwrap();

    return match db.create_world() {
        Ok(()) => {
            HttpResponse::Ok().body("OK")
        },

        Err(e) => {
            HttpResponse::BadRequest().body(e.to_string())
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let path = std::env::current_dir().unwrap().join("db");
    let mut db = match path.exists() {
        true => {
            println!("Retrieving existing database");
            Database::from(path).expect("Something went wrong getting the existing database")
        },
        false => {
            println!("Creating new database");
            Database::new(path).expect("Something went wrong creating a new database")
        },
    };

    // HttpServer server spawns "workers" equal to the number
    // of phyiscal cpu's / cores available in the system
    //
    // the db is created outside of HttpServer::new so it is only created once,
    // we have to clone the db into each HttpServer
    HttpServer::new(move || {

        App::new()
        .app_data(web::Data::new(APIData {
            // database: Mutex::new(db.clone()),
            database: Mutex::new(&mut db),
            auth: fs::read_to_string("auth.txt").expect("Auth setup failed")
        }))
            .service(hello)
            .service(stats)
            .service(get_current_world)
            .service(get_db_path)
            .service(create_world)
            .service(switch_world)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}