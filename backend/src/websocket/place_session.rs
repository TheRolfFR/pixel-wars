use std::io::Write;

use actix_web_actors::ws;
use actix::prelude::*;

use crate::model;

use super::place_server::{ConnectMessage, DisconnectMessage, OnlineUserCountMessage, PlaceServer};

/// Web socket place session
pub struct PlaceSession {
    pub uuid: String,
    /// Place server
    pub place_server: Addr<PlaceServer>,
}

impl Actor for PlaceSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("User #{} connected", self.uuid);
        let addr = ctx.address();
        self.place_server.send(ConnectMessage {
            addr,
            author_uuid: self.uuid.clone(),
        })
        .into_actor(self)
        .then(|res, _, ctx| {
            match res {
                Ok(_) => (),
                // something is wrong with chat server
                _ => ctx.stop(),
            }
            fut::ready(())
        })
        .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        log::info!("User #{} disconnecting", self.uuid);
        self.place_server.do_send(DisconnectMessage {
            author_uuid: self.uuid.clone()
        });
        Running::Stop
    }
}

impl Handler<OnlineUserCountMessage> for PlaceSession {
    type Result = ();

    fn handle(&mut self, msg: OnlineUserCountMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(format!("/count {}", msg.0));
    }
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

impl Handler<model::PixelColorUpdateMessage> for PlaceSession {
    type Result = ();

    fn handle(&mut self, msg: model::PixelColorUpdateMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.binary(msg.serialize());
    }
}

impl PlaceSession {
    pub fn place_pixel(&mut self, bin: &[u8], ctx: &mut ws::WebsocketContext<PlaceSession>) -> Result<(), ()>
    {
        let pixel_update = model::PixelColorUpdateMessage::deserialize(bin).map_err(|_| ())?;

        let user_pixel_update = model::UserPixelColorMessage {
            pixel_update,
            uuid: self.uuid.clone()
        };

        self.place_server.send(user_pixel_update)
        .into_actor(self)
        .then(|res, _, ctx| {
            let opt_err = res
                .map_err(|e| e.to_string())
                .and_then(|inner| inner)
                .err();

            if let Some(err) = opt_err {
                ctx.text(err);
            }

            fut::ready(())
        })
        .wait(ctx);

        Ok(())
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PlaceSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => {
                self.place_pixel(&bin[..], ctx).ok();
            },
            _ => (),
        }
    }
}
