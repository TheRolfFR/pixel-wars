use std::time::{Instant, SystemTime, UNIX_EPOCH};

use actix_web::{cookie::{self, time::Duration, CookieBuilder, SameSite}, error::{self, InternalError}, get, web, HttpRequest, HttpResponse, Responder, ResponseError};
use redis::AsyncCommands;
use redis::RedisResult;
use serde::Serialize;

use crate::model::{self, BackendError};

const COOKIE_NAME: &str = "sessionUUID";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClientTimeoutResponse {
    last_timestamp: usize,
    remaining_pixels: usize
}

pub async fn client_timeout(
    req: HttpRequest,
    redis: web::Data<redis::Client>,
    config: web::Data<model::Config>
) -> actix_web::Result<impl Responder> {
    let cookie = req.cookie(COOKIE_NAME).ok_or(error::ErrorBadRequest("No cookie provided"))?;
    let uuid = cookie.value().to_string();

    let mut con  = redis.get_multiplexed_async_connection().await
        .map_err(BackendError::from)?;

    let client_string: String = con.get(uuid.to_string()).await
        .map_err(BackendError::from)?;

    let mut client: model::Client = serde_json::from_str(&client_string)
        .map_err(BackendError::from)?;


    let start = SystemTime::now();
    let current_timestamp = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let last_timestamp = Duration::seconds_f64(client.last_timestamp);
    let duration = current_timestamp - last_timestamp;

    dbg!(&current_timestamp, &last_timestamp, duration.abs());
    if duration.abs() > Duration::MINUTE {
        client.remaining_pixels = config.pixels_per_minute;
        client.last_timestamp = current_timestamp.as_secs_f64();

        let client_string = client.encode_json()
            .map_err(BackendError::from)?;

        con.set(uuid, client_string).await
            .map_err(BackendError::from)?;
    }

    Ok(HttpResponse::Ok().json(ClientTimeoutResponse {
        last_timestamp: client.last_timestamp.ceil() as usize,
        remaining_pixels: client.remaining_pixels
    }))
}

pub async fn profiles_add(
    req: HttpRequest,
    redis: web::Data<redis::Client>,
    profile: web::Json<model::Profile>
) -> actix_web::Result<impl Responder> {
    let uuid = req.cookie(COOKIE_NAME)
        .ok_or(error::ErrorBadRequest("No cookie provided"))?
        .value().to_string();

    let mut con  = redis.get_multiplexed_async_connection().await
        .map_err(BackendError::from)?;

    let mut client: model::Client = serde_json::from_str(&
            con.get::<_,String>(&uuid).await
            .map_err(BackendError::from)?
        )
        .map_err(BackendError::from)?;

    if client.profile.is_some() {
        return Err(error::ErrorNotAcceptable("User already has a profile"));
    }

    client.profile = Some(profile.into_inner());
    let client_string = client.encode_json()
        .map_err(BackendError::from)?;

    con.set(&uuid, client_string).await
        .map_err(BackendError::from)?;

    Ok(HttpResponse::Ok())
}

pub async fn profiles_get(
    req: HttpRequest,
    redis: web::Data<redis::Client>
) -> actix_web::Result<impl Responder> {
    let uuid = req.cookie(COOKIE_NAME)
        .ok_or(error::ErrorBadRequest("No cookie provided"))?
        .value().to_string();

    let mut con  = redis.get_multiplexed_async_connection().await
        .map_err(BackendError::from)?;

    let client_string: String = con.get(&uuid).await
        .map_err(BackendError::from)?;

    let client: model::Client = serde_json::from_str(&client_string)
        .map_err(BackendError::from)?;

    let profile = client.profile
        .ok_or(error::ErrorNotFound("Client doesn't have a profile..."))?;

    Ok(HttpResponse::Ok().json(profile))
}
