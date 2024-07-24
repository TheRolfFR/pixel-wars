use std::time::{Instant, SystemTime, UNIX_EPOCH};

use actix_web::{cookie::{self, time::Duration, CookieBuilder, SameSite}, error, get, web, HttpRequest, HttpResponse, Responder};
use redis::AsyncCommands;
use redis::RedisResult;
use uuid::Uuid;

use crate::model::{self, BackendError};

const COOKIE_NAME: &str = "sessionUUID";

pub async fn session_get(
    req: HttpRequest,
    redis: web::Data<redis::Client>,
    config: web::Data<model::Config>
) -> actix_web::Result<HttpResponse> {
    let mut con = redis.get_multiplexed_async_connection().await.map_err(BackendError::from)?;
    if let Some(uuid) = req.cookie(COOKIE_NAME).map(|u| u.value().to_string()) {
        let redis_res: RedisResult<String> = con.get(uuid).await;
        if redis_res.is_ok() {
            return Ok(HttpResponse::Ok().into());
        }
    } // return if already redis entry to cookie uuid

    // create new uuid
    let new_uuid = Uuid::new_v4().to_string();

    // create new cookie
    let host = req.uri().host().unwrap_or("localhost");
    let hostname = host.split(':').next().unwrap();
    let cookie = CookieBuilder::new(COOKIE_NAME, new_uuid.clone())
        .same_site(SameSite::Strict)
        .max_age(Duration::days(400)) //max-age = 400 days, maximum allowed by chrome
        .path("/")
        .domain(hostname)
        .secure(false)
        .http_only(true)
        .finish();

    // create client with last seen timestamp
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let in_seconds = since_the_epoch.as_secs_f64() * 1000f64;
    let client = model::Client {
        profile: None,
        last_timestamp: in_seconds,
        remaining_pixels: config.pixels_per_minute
    };
    // send client to redis
    let client_string: String = client.encode_json().map_err(BackendError::from)?;
    log::info!("Added user UUID={} with value: {:?}", &new_uuid, &client_string);
    con.set(new_uuid, client_string).await.map_err(BackendError::from)?;

    // respond with cookie
    let res = HttpResponse::Ok().cookie(cookie).finish();
    Ok(res)
}
