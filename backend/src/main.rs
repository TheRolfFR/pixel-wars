use std::path::Path;
use std::env;

use redis;

use actix::Actor;
use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use actix_files as fs;


use backend::{actors, dev, model, routes::routes};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));


    // TODO: extract args better
    let mut args = std::env::args();
    let config_path_string = args.nth(1).unwrap_or("../config.json".into());

    let config_path = Path::new(&config_path_string);
    let absolute_path = std::path::absolute(config_path)
        .map_err(|e| format!("Failed to create absolute path : {e}")).unwrap();

    // canvas config
    log::info!("Opening config file located at {}...", absolute_path.display());
    let config = {
        let mut config = model::Config::from_file(absolute_path).expect("config.json file not found");
        // overwrite with env
        if let Ok(env_var) = env::var("REDIS_URL") {
            config.redis_url = env_var;
        }
        if let Ok(env_var) = env::var("HOST") {
            config.host = env_var;
        }
        if let Ok(env_var) = env::var("PORT") {
            config.port = env_var.parse().expect("Failed to extract PORT from ENV");
        }
        config
    };

    // real-time db config
    let redis_url = config.redis_url.clone();
    log::info!("Starting redis on {}", &redis_url);
    let redis_client = redis::Client::open(redis_url).unwrap();

    // place server
    let server = actors::PlaceServer::new(redis_client.clone(), config.clone()).start();

    // http server config
    let ip = config.host.clone();
    let port = config.port.clone();
    let devmode = cfg!(debug_assertions);
    log::info!("starting HTTP server at http://{ip}:{port}");
    if devmode {  log::info!("Adding reverse proxy for frontend..."); }

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        let mut app = App::new()
            .wrap(cors)
            .app_data(web::Data::new(server.clone()))
            // .app_data(web::JsonConfig::default().limit(1024)) // <- limit size of the payload (global configuration)
            .app_data(web::Data::new(redis_client.clone())) // db connection
            .app_data(web::Data::new(config.clone())) // canvas config
            // .wrap(actix_web::middleware::Logger::new("%a \"%r\" %s %b \"%{Referer}i\" %T")) // log things to stdout
            .configure(routes);

        if devmode {
            app = app.configure(dev::add_reverse_proxy);
        } else {
            app = app
            .service(fs::Files::new("/favicons", "../frontend/public/favicons"))
            .service(fs::Files::new("/assets", "../frontend/dist/assets"))
            .service(fs::Files::new("/", "../frontend/dist/").index_file("index.html"));
        }

        app
    })
    .bind((ip, port))?
    .run()
    .await
}
