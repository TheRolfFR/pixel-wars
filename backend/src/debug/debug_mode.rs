use actix_web::web;

use crate::debug::proxy_handler;

pub fn add_reverse_proxy(
    cfg: &mut web::ServiceConfig,
) {
    cfg.default_service(web::to(proxy_handler));
}
