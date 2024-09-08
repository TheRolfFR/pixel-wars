use std::time::{Instant, SystemTime, UNIX_EPOCH};

use actix_web::{cookie::{self, time::Duration, CookieBuilder, SameSite}, error::{self, InternalError}, get, web, HttpRequest, HttpResponse, Responder, ResponseError};
use redis::AsyncCommands;
use redis::RedisResult;
use serde::Serialize;

use crate::model::{self, BackendError, Client};

const COOKIE_NAME: &str = "sessionUUID";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClientTimeoutResponse {
    last_timestamp: u64,
    next_timestamp: u64,
    timeout: u64,
    remaining_pixels: usize
}

#[get("/client/timeout")]
pub async fn client_timeout(
    req: HttpRequest,
    redis: web::Data<redis::Client>,
    config: web::Data<model::Config>
) -> actix_web::Result<impl Responder> {
    let cookie = req.cookie(COOKIE_NAME).ok_or(error::ErrorBadRequest("No cookie provided"))?;
    let uuid = cookie.value().to_string();

    let mut con  = redis.get_multiplexed_async_connection().await
        .map_err(BackendError::from)?;

    let redis_result = con.get::<String, String>(uuid.to_string()).await;
    let mut client = Client::from_redis(redis_result, config.base_pixel_amount);

    let current_timestamp = model::Client::timestamp_now();
    let duration_secs = current_timestamp - client.last_timestamp;
    let timeout_secs = config.timeout.as_secs();


    if timeout_secs > 0 && duration_secs >= timeout_secs - 1 {
        client.remaining_pixels = config.base_pixel_amount;
        client.last_timestamp = current_timestamp;

        let client_string = client.encode_json()
            .map_err(BackendError::from)?;

        con.set(uuid, client_string).await
            .map_err(BackendError::from)?;
    }

    Ok(HttpResponse::Ok().json(ClientTimeoutResponse {
        last_timestamp: client.last_timestamp,
        remaining_pixels: client.remaining_pixels,
        timeout: timeout_secs,
        next_timestamp: client.last_timestamp + timeout_secs,
    }))
}
