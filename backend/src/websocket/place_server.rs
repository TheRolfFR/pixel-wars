use std::{collections::HashMap, time::{Duration, SystemTime, UNIX_EPOCH}};

use actix::prelude::*;
use redis::{self, Commands};

use crate::{controller::canvas_redis_set, model::{self, PixelColorUpdateMessage}};

use super::place_session::PlaceSession;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Debug, Clone)]
pub struct PlaceServer {
    config: model::Config,
    redis_client: redis::Client,
    sessions: HashMap<String, Addr<PlaceSession>>,
}

/// New session is created
#[derive(Message)]
#[rtype(usize)]
pub struct ConnectMessage {
    pub author_uuid: String,
    pub addr: Addr<PlaceSession>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct DisconnectMessage {
    pub author_uuid: String,
}

/// Online user count
#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct OnlineUserCountMessage(pub usize);

impl PlaceServer {
    pub fn new(redis_client: redis::Client, config: model::Config) -> Self {
        Self {
            config,
            redis_client,
            sessions: HashMap::new()
        }
    }
}

impl PlaceServer {
    fn send_online(&self, message_count: OnlineUserCountMessage)
    {
        for session in self.sessions.values() {
            session.do_send(message_count.clone());
        }
    }

    fn send_pixel_update(&self, msg: PixelColorUpdateMessage)
    {
        for session in self.sessions.values() {
            session.do_send(msg.clone());
        }
    }
}

impl Actor for PlaceServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

impl Handler<ConnectMessage> for PlaceServer {
    type Result = usize;

    fn handle(&mut self, msg: ConnectMessage, _: &mut Context<Self>) -> Self::Result {
        self.sessions.insert(msg.author_uuid.clone(), msg.addr);

        let count = self.sessions.len();
        let message_count = OnlineUserCountMessage(count);
        self.send_online(message_count);

        count
    }
}

impl Handler<DisconnectMessage> for PlaceServer {
    type Result = ();

    fn handle(&mut self, msg: DisconnectMessage, _: &mut Context<Self>) -> Self::Result {
        self.sessions.remove(&msg.author_uuid);

        let count = self.sessions.len();
        let message_count = OnlineUserCountMessage(count);
        self.send_online(message_count);
    }
}

impl Handler<model::UserPixelColorMessage> for PlaceServer {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: model::UserPixelColorMessage, _ctx: &mut Self::Context) -> Self::Result {
        // log::info!("Received new pixel color message: {:?}", &msg);

        let mut con = self.redis_client.get_connection()
            .map_err(|e| e.to_string())?;
        let uuid = msg.uuid;
        let pixel_update = msg.pixel_update;
        let config = &self.config;

        if pixel_update.pos_x > config.canvas_width || pixel_update.pos_y > config.canvas_height {
            return Err("Invalid position in canvas".to_string())
        }


        // get client
        let client_string: String = con.get(uuid.clone())
            .map_err(|e| e.to_string())?;
        let mut client: model::Client = serde_json::from_str(&client_string)
            .map_err(|e| e.to_string())?;
        let start = SystemTime::now();
        let current_timestamp = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let last_timestamp = Duration::from_secs_f64(client.last_timestamp);
        let duration = current_timestamp - last_timestamp;


        // update client
        if duration > Duration::from_secs(60) {
            client.remaining_pixels = config.pixels_per_minute;
            client.last_timestamp = current_timestamp.as_secs_f64();
        }
        if client.remaining_pixels == 0 {
            return Err("No pixels left".to_string());
        }
        client.remaining_pixels -= 1;
        let client_string = client.encode_json()
            .map_err(|e| e.to_string())?;
        con.set(uuid, client_string)
            .map_err(|e| e.to_string())?;

        // update db
        canvas_redis_set(&self.redis_client, config, &pixel_update)?;


        // notify sessions
        self.send_pixel_update(pixel_update);

        Ok(())
    }
}
