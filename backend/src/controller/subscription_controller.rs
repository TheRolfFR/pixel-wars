use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};

use actix::{Actor, StreamHandler};
use actix_web::{error, http::{Error, StatusCode}, web, App, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError};
use actix_web_actors::ws::{self, WebsocketContext};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use bytes::Bytes;

use crate::model::{self, BackendError, SESSION_COOKIE_NAME};

/// Define HTTP actor
struct MyWs {
    config: model::Config,
    uuid: String,
    con: MultiplexedConnection,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

async fn handle_binary(bin: Bytes, my_ws: &mut MyWs) -> Result<(), ()>
{
    let pixel_update: model::PixelColorUpdateMessage = bincode::deserialize(&bin[..]).map_err(|_| ())?;

    let config = &my_ws.config;
    let mut con = my_ws.con.clone();
    let uuid = my_ws.uuid.clone();

    if pixel_update.pos_x > config.canvas_width || pixel_update.pos_y > config.canvas_height {
        return Err(())
    }
    
    let client_string: String = con.get(uuid.clone()).await
        .map_err(|_| ())?;

    let mut client: model::Client = serde_json::from_str(&client_string)
        .map_err(|_| ())?;

    let start = SystemTime::now();
    let current_timestamp = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let last_timestamp = Duration::from_secs_f64(client.last_timestamp);
    let duration = current_timestamp - last_timestamp;

    if duration > Duration::from_secs(60) {
        client.remaining_pixels = config.pixels_per_minute;
        client.last_timestamp = current_timestamp.as_secs_f64();
    }

    if client.remaining_pixels == 0 {
        return Err(());
    }

    client.remaining_pixels -= 1;
    let client_string = client.encode_json()
        .map_err(|_| ())?;

    con.set(uuid, client_string).await
        .map_err(|_| ())?;

    con.publish("changes", &bin[..]).await
        .map_err(|_| ())?;

    Ok(())
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => {
                handle_binary(bin.clone(), self).await.ok();
                ctx.binary(bin)
            },
            _ => (),
        }
    }
}

pub async fn subscription_get(
    req: HttpRequest,
    redis: web::Data<redis::Client>,
    stream: web::Payload
) -> actix_web::Result<HttpResponse> {
    let uuid = req.cookie(SESSION_COOKIE_NAME)
        .ok_or(error::ErrorBadRequest("No cookie provided"))?
        .value().to_string();

    let mut con = redis.get_multiplexed_async_connection().await
        .map_err(BackendError::from)?;

    con.get::<_,Option<String>>(&uuid).await
        .map_err(BackendError::from)?;

    ws::start(MyWs {
        config: req.app_data::<model::Config>().unwrap().clone(),
        uuid,
        con
    }, &req, stream)
}
