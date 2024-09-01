use actix_web::web;
use crate::controller::*;

pub fn routes(cfg: &mut web::ServiceConfig) {
    let api_scope = web::scope("/api")
        .service(web::resource("/canvas").route(web::get().to(canvas_get)))
        .service(web::resource("/canvas/{offset}/{color}").route(web::put().to(canvas_update)))
        .service(web::resource("/getSession").route(web::get().to(session_get)))
        .service(web::resource("/client/details").route(web::get().to(client_timeout)))
        .service(web::resource("/subscribe").route(web::get().to(subscription_get)))
        .service(web::resource("/profiles/new").route(web::post().to(profiles_add)))
        .service(web::resource("/profiles/get").route(web::get().to(profiles_get)))
        ;
    cfg.service(api_scope);
}
