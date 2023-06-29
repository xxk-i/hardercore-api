use actix_web::{web, get, Responder, HttpResponse};
use std::{sync::Mutex, time};

use super::info::APIData;

#[get("/")]
pub async fn hello() -> impl Responder {
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

#[get("/sleep")]
pub async fn sleep() -> impl Responder {
    std::thread::sleep(time::Duration::from_secs(5));
    HttpResponse::Ok()
}

#[get("/world/stats/{uuid}")]
pub async fn get_stats(data: web::Data<Mutex<APIData>>, path: web::Path<(String,)>) -> impl Responder {
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
pub async fn get_all_stats(data: web::Data<Mutex<APIData>>) -> impl Responder {
    let db = &data.lock().unwrap().database;

    HttpResponse::Ok().body(db.get_all_stats().expect("Saved player stats JSON is probably malformed!"))
}

#[get("world/uptime")]
pub async fn get_uptime(data: web::Data<Mutex<APIData>>) -> impl Responder {
    let db = &data.lock().unwrap().database; 
    
    HttpResponse::Ok().body(db.world.uptime.to_string())
}