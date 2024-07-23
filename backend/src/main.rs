use std::sync::Mutex;

use backend::{model::{self, Canvas}, routes::routes};

use redis;
use actix_web::{web, App, HttpServer, middleware};

const REDIS_CONNECTION_STRING: &str = "redis://172.18.115.69/";
const WEB_IP: &str = "127.0.0.1";
const WEB_PORT: u16 = 5173;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));


    // TODO: extract args better
    let mut args = std::env::args();
    let config_path = args.nth(1).unwrap_or("..\\config.json".into());


    // http server config
    let (ip, port) = (WEB_IP, WEB_PORT);
    log::info!("starting HTTP server at http://{}:{}", ip, port);

    // real-time db config
    let redis = redis::Client::open(REDIS_CONNECTION_STRING).unwrap();
    // canvas config
    let config = model::Config::from_file(config_path).unwrap();

    // Shared mutanle application state
    let app_state = web::Data::new(model::BackendAppState {
        canvas_valid: Mutex::new(Canvas {
            colors: vec![],
            valid: false
        })
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            // .app_data(web::JsonConfig::default().limit(1024)) // <- limit size of the payload (global configuration)
            .app_data(web::Data::new(redis.clone())) // db connection
            .app_data(web::Data::new(config.clone())) // canvas config
            .wrap(middleware::Logger::default()) // log things to stdout
            .configure(routes)
    })
    .bind((ip, port))?
    .run()
    .await
}
