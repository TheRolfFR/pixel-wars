use std::{io::Write, time::Instant};

use actix_ws::{self as ws, Session};

use actix::prelude::*;

use crate::model;

use super::place_server::PlaceServer;
use super::messages::ServerInternalMessage;

/// Web socket place session
#[derive(Clone)]
pub struct PlaceSession {
    pub uuid: String,
    /// Place server
    pub place_server: Addr<PlaceServer>,
    pub close_reason: Option<ws::CloseReason>,
    pub start: Instant,
}

impl Actor for PlaceSession {
    type Context = Context<Self>;
}

impl model::PixelColorUpdateMessage {
    pub fn deserialize(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 5 {
            return Err("Error deserializing pixel color update");
        }

        let pos_x = u16::from_be_bytes([data[0], data[1]]);
        let pos_y = u16::from_be_bytes([data[2], data[3]]);
        let color =  data[4];
        Ok(Self {
            pos_x,
            pos_y,
            color
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.write_all(&self.pos_x.to_be_bytes()).unwrap();
        buffer.write_all(&self.pos_y.to_be_bytes()).unwrap();
        buffer.write_all(&self.color.to_be_bytes()).unwrap();
        buffer
    }
}

impl PlaceSession {
    pub fn new(uuid: String, place_server: Addr<PlaceServer>) -> Self {
        Self {
            uuid,
            place_server,
            close_reason: None,
            start: Instant::now(),
        }
    }
    pub async fn place_pixel(&self, bin: &[u8]) -> Result<(), String>
    {
        let user_pixel_update = model::UserPixelColorMessage {
            pixel_update: model::PixelColorUpdateMessage::deserialize(bin).map_err(|e| e.to_string())?,
            uuid: self.uuid.clone()
        };

        self.place_server.send(user_pixel_update).await.map_err(|e| e.to_string())?.ok();

        Ok(())
    }

    pub async fn handle_ws(
        &self,
        msg: ws::Message,
        mut session: ws::Session
    ) -> Option<Option<ws::CloseReason>> {
        match msg {
            ws::Message::Ping(msg) => {
                session.pong(&msg).await.ok();
                None
            },
            ws::Message::Text(text) => {
                session.text(text).await.ok();
                None
            },
            ws::Message::Binary(bin) => {
                if let Err(e) = self.place_pixel(&bin[..]).await {
                    session.text(e).await.ok();
                }
                None
            },
            ws::Message::Close(reason) => {
                Some(reason)
            }
            _ => None,
        }
    }

    pub async fn handle_message(&self, int_message: ServerInternalMessage, mut session: Session) -> () {
        match int_message {
            ServerInternalMessage::Pixel(pixel_update) => {
                session.binary(pixel_update.serialize()).await.ok();
            },
            ServerInternalMessage::Online(count) => {
                session.text(format!("/count {count}")).await.ok();
            }
        };
    }
}
