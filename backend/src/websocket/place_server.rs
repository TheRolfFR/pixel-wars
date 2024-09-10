use std::collections::HashMap;

use redis::{self, Commands};

use actix_ws as ws;
use actix::prelude::*;

use crate::{controller::canvas_redis_set, model::PixelColorUpdateMessage};
use crate::model;

use super::messages::{ConnectMessage, DisconnectMessage, OnlineUserCountMessage};
use super::PlaceSession;

pub struct PlaceServer {
    config: model::Config,
    redis_client: redis::Client,
    sessions: HashMap<String, Addr<PlaceSession>>,
}
impl Actor for PlaceServer {
    type Context = Context<Self>;
}

impl PlaceServer {
    pub fn new(redis_client: redis::Client, config: model::Config) -> Self {
        Self {
            config,
            redis_client,
            sessions: HashMap::new()
        }
    }
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

impl Handler<ConnectMessage> for PlaceServer {
    type Result = ();

    fn handle(&mut self, msg: ConnectMessage, _: &mut Context<Self>) -> Self::Result {
        log::info!("Starting PlaceSession for #{}", msg.uuid.clone());
        self.sessions.insert(msg.uuid, msg.addr);

        let message_count = OnlineUserCountMessage(self.sessions.len());
        self.send_online(message_count);
    }
}

impl Handler<DisconnectMessage> for PlaceServer {
    type Result = ();

    fn handle(&mut self, msg: DisconnectMessage, _ctx: &mut Context<Self>) -> Self::Result {
        match match msg.close_reason {
            Some(reason) => { match reason.code {
                    ws::CloseCode::Normal | ws::CloseCode::Away => { None },
                    _ => { Some(reason) }
            } },
            None => {
                None
            }
        } {
            Some(reason) => {
                log::info!("Stopping PlaceSession for #{} after {:?} with reason {}", msg.uuid.clone(), msg.elapsed, u16::from(reason.code));
            },
            None => {
                log::info!("Stopping PlaceSession for #{} after {:?} without reason", msg.uuid.clone(), msg.elapsed);
            }
        };
        self.sessions.remove(&msg.uuid);

        let count = self.sessions.len();
        let message_count = OnlineUserCountMessage(count);
        self.send_online(message_count);
    }
}

impl Handler<model::UserPixelColorMessage> for PlaceServer {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: model::UserPixelColorMessage, _ctx: &mut Context<Self>) -> Self::Result {
        // log::info!("Received new pixel color message: {:?}", &msg);

        let mut con = self.redis_client.get_connection()
            .map_err(|e| e.to_string())?;
        let uuid = msg.uuid;
        let pixel_update = msg.pixel_update;
        let config = &self.config;

        if pixel_update.pos_x > (config.canvas_width as u16) || pixel_update.pos_y > (config.canvas_height as u16) {
            return Err("Invalid position in canvas".to_string())
        }


        // get client
        let redis_result = con.get::<String, String>(uuid.clone());
        let mut client = model::Client::from_redis(redis_result, config.base_pixel_amount);

        let current_timestamp = model::Client::timestamp_now();
        let duration_secs = current_timestamp - client.last_timestamp;
        let timeout_secs = config.timeout.as_secs();

        // update client
        // agree with 1s margin
        if client.remaining_pixels == 0 && duration_secs >= timeout_secs - 1 {
            client.remaining_pixels = config.base_pixel_amount;
            client.last_timestamp = current_timestamp;
        }

        // reduce pixel number
        if client.remaining_pixels == 0 {
            return Err("No pixels left".to_string());
        } else {
            client.remaining_pixels -= 1;
        }

        // save client
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
