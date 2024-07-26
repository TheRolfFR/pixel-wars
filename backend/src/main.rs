use std::sync::Mutex;

use backend::{debug::add_reverse_proxy, model::{self, Canvas}, routes::routes};
use actix_cors::Cors;

use redis;
use actix_web::{web, App, HttpServer};

const REDIS_CONNECTION_STRING: &str = "redis://172.18.115.69/";
const DEBUG_WEB_IP: &str = "127.0.0.1";
const DEBUG_WEB_PORT: u16 = 8080;
const PROD_WEB_IP: &str = "0.0.0.0";
const PROD_WEB_PORT: u16 = 80;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));


    // TODO: extract args better
    let mut args = std::env::args();
    let config_path = args.nth(1).unwrap_or("..\\config.json".into());

    // canvas config
    let config = model::Config::from_file(config_path).unwrap();

    // real-time db config
    let redis_url = config.redis_url.clone().unwrap_or(REDIS_CONNECTION_STRING.into());
    log::info!("Starting redis on {}", &redis_url);
    let redis = redis::Client::open(redis_url).unwrap();

    // Shared mutanle application state
    let app_state = web::Data::new(model::BackendAppState {
        canvas_valid: Mutex::new(Canvas {
            colors: vec![0; (config.canvas_width as usize) * (config.canvas_height as usize) / 2],
            valid: false
        }),
    });


    dbg!(app_state.canvas_valid.lock().unwrap().colors.len());


    // http server config
    let (ip, port) = if config.debug_mode {
        (DEBUG_WEB_IP, DEBUG_WEB_PORT)
    } else {
        (PROD_WEB_IP, PROD_WEB_PORT)
    };
    log::info!("starting HTTP server at http://{}:{}", ip, port);
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        let mut app = App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            // .app_data(web::JsonConfig::default().limit(1024)) // <- limit size of the payload (global configuration)
            .app_data(web::Data::new(redis.clone())) // db connection
            .app_data(web::Data::new(config.clone())) // canvas config
            // .wrap(actix_web::middleware::Logger::new("%a \"%r\" %s %b \"%{Referer}i\" %T")) // log things to stdout
            .configure(routes);

        if config.debug_mode {
            app = app.configure(add_reverse_proxy);
        }

        app
    })
    .bind((ip, port))?
    .run()
    .await
}
