use actix_web::{web, put, Responder, HttpResponse};
use std::{sync::Mutex};

use super::util::APIData;
use super::util::SwitchInfo;
use super::util::Info;

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

#[put("/world/stats/{uuid}")]
pub async fn stats(data: web::Data<Mutex<APIData>>, info: web::Json<Info>, path: web::Path<(String,)>) -> impl Responder {
    let mut apidata = data.lock().unwrap();
    
    if !info.auth.eq(&apidata.auth) {
        return HttpResponse::Forbidden().body("Auth failed")
    }

    let db = &mut apidata.database;

    let uuid = path.into_inner().0;

    match db.world.merge_stats(uuid, info.into_inner()) {
        Ok(_) => {},

        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };

    HttpResponse::Ok().body("OK")
}
