use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};

use actix::{Actor, Addr, StreamHandler};
use actix_web::{error, http::{Error, StatusCode}, web, App, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError};
use actix_web_actors::ws::{self, WebsocketContext};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use bytes::Bytes;

use crate::{model::{self, BackendError, SESSION_COOKIE_NAME}, websocket::{PlaceServer, PlaceSession}};

pub async fn subscription_get(
    req: HttpRequest,
    redis: web::Data<redis::Client>,
    stream: web::Payload,
    server: web::Data<Addr<PlaceServer>>,
) -> actix_web::Result<HttpResponse> {
    let uuid = req.cookie(SESSION_COOKIE_NAME)
        .ok_or(error::ErrorBadRequest("No cookie provided"))?
        .value().to_string();

    let mut con = redis.get_multiplexed_async_connection().await
        .map_err(BackendError::from)?;

    con.get::<_,Option<String>>(&uuid).await
        .map_err(BackendError::from)?;

    log::info!("Starting PlaceSession for #{}", uuid);
    ws::start(PlaceSession {
        uuid: uuid,
        place_server: server.get_ref().clone()
    }, &req, stream)
}