#![allow(unused_variables)]
use actix_web::{get, post, web, HttpResponse, Responder};
use redis;
use crate::controller::*;

#[get("/subscribe")]
async fn subscription_get(redis: web::Data<redis::Client>) -> actix_web::Result<impl Responder> {
    Ok(HttpResponse::Ok())
}

#[post("/profiles/new")]
async fn profiles_add(redis: web::Data<redis::Client>) -> actix_web::Result<impl Responder> {
    Ok(HttpResponse::Ok())
}

#[get("/profiles/get")]
async fn profiles_get(redis: web::Data<redis::Client>) -> actix_web::Result<impl Responder> {
    Ok(HttpResponse::Ok())
}

#[get("/client/details")]
async fn client_timeout(redis: web::Data<redis::Client>) -> actix_web::Result<impl Responder> {
    Ok(HttpResponse::Ok())
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    let api_scope = web::scope("/pixelwars/api")
        .service(web::resource("/canvas").route(web::get().to(canvas_get)))
        .service(web::resource("/canvas/{offset}/{color}").route(web::put().to(canvas_update)))
        .service(web::resource("/getSession").route(web::get().to(session_get)))
        .service(profiles_add)
        .service(profiles_get)
        .service(client_timeout)
        ;
    cfg.service(api_scope);
}
