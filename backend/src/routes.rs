use actix_web::web;
use crate::controller::*;

pub fn routes(cfg: &mut web::ServiceConfig) {
    let api_scope = web::scope("/api")
        .service(canvas_get)
        .service(session_get)
        .service(client_timeout)
        ;

    cfg
        .service(api_scope)
        .service(websocket_start)
        ;
}
