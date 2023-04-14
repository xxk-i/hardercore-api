use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, put};
use serde::Deserialize;

mod db;
use db::Database;

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct SwitchInfo {
    world: u64
}

#[derive(Deserialize)]
#[allow(non_snake_case)] // we use json naming convention here
struct Info {
    UUID: String,
    timeInWater: Option<u64>,
    damageTaken: Option<u64>,
    hasDied: Option<bool>,
    mobsKilled: Option<u64>,
    foodEaten: Option<u64>
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("You have reached the homepage of hardercore-api")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[get("/world/current")]
pub async fn get_current_world() -> impl Responder {
    let db: Database = Database::new(std::env::current_dir().unwrap()).expect("Something went wrong creating the database");
    db.get_current_world()
}

#[get("database/path")]
pub async fn get_db_path() -> impl Responder {
    let db: Database = Database::new(std::env::current_dir().unwrap()).expect("Something went wrong creating the database");
    db.get_path() 
}

#[put("/world/stats")]
async fn water(info: web::Json<Info>) -> impl Responder {
    if let Some(time) = &info.timeInWater {
        Database::update_time_in_water(info.UUID.clone(), *time);
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

    HttpResponse::Ok()
}

#[put("/world")]
async fn switch_world(switch_info: web::Json<SwitchInfo>) -> impl Responder {
    let mut db = Database::new(std::env::current_dir().unwrap()).unwrap();

    return match db.switch_world(switch_info.world) {
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
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .service(water)
            .service(get_current_world)
            .service(get_db_path)
            .service(switch_world)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}