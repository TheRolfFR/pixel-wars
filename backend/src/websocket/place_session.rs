use actix_web_actors::ws;
use actix::prelude::*;

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

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PlaceSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => {
                // handle_binary(bin.clone(), self).await.ok();
                ctx.binary(bin)
            },
            _ => (),
        }
    }
}
