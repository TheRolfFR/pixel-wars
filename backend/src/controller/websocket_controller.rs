use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};

use tokio::task::spawn_local;
use actix::{Actor, Addr, StreamHandler};
use actix_web::{error, http::{Error, StatusCode}, web, App, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError, get};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use bytes::Bytes;

use crate::{model::{self, BackendError, SESSION_COOKIE_NAME}, actors::{handler::handle_ws, PlaceServer, PlaceSession}};

#[get("/websocket")]
pub async fn websocket_start(
    req: HttpRequest,
    body: web::Payload,
    redis: web::Data<redis::Client>,
    server: web::Data<Addr<PlaceServer>>,
) -> actix_web::Result<HttpResponse> {
    let uuid = req.cookie(SESSION_COOKIE_NAME)
        .ok_or(error::ErrorBadRequest("No cookie provided"))?
        .value().to_string();

    let mut con = redis.get_multiplexed_async_connection().await
        .map_err(BackendError::from)?;

    con.exists::<_,()>(&uuid).await
        .map_err(BackendError::from)?;

    let (response, session, msg_stream) = actix_ws::handle(&req, body)?;

    spawn_local(async move {
        handle_ws(uuid, session, server.get_ref().clone(), msg_stream).await;
    });

    Ok(response)
}
