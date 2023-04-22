use actix_web::{web::{self}, App, HttpServer};
use std::{sync::Mutex, fs};

mod db;
mod putters;
mod getters;

mod util;
use util::APIData;

use db::Database;

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
            .service(putters::stats)
            .service(putters::create_world)
            .service(putters::switch_world)
            .service(getters::get_all_stats)
            .service(getters::get_stats)
            .service(getters::get_current_world)
            .service(getters::get_db_path)
    })
    .bind(("127.0.0.1", 8080))?
    .bind(("172.25.254.1", 8080))?
    .run()
    .await

}