use std::time::Instant;

use actix_ws::{self as ws, CloseReason, Session};

use actix::prelude::*;

use crate::model;

use super::place_server::PlaceServer;
use super::messages::{ConnectMessage, DisconnectMessage, OnlineUserCountMessage, UserPixelColorMessage, StopSession, WsMessage};

pub struct PlaceSession {
    uuid: String,
    /// Place server
    place_server: Addr<PlaceServer>,
    start: Instant,
    session: Session,
    close_reason: Option<ws::CloseReason>,
}

impl PlaceSession {
    pub fn new(uuid: String, place_server: Addr<PlaceServer>, session: Session) -> Self {
        Self {
            uuid,
            place_server,
            start: Instant::now(),
            session,
            close_reason: None,
        }
    }

    fn close(&mut self, msg: Option<CloseReason>, ctx: &mut Context<Self>) {
        self.close_reason = msg.clone();
        let session = self.session.clone();
        async move {
            session.close(msg).await.ok();
        }
        .into_actor(self)
        .wait(ctx);
        ctx.stop();
    }
}

impl Actor for PlaceSession {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.place_server.do_send(ConnectMessage {
            uuid: self.uuid.clone(),
            addr: ctx.address()
        });
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        self.place_server.do_send(DisconnectMessage {
            uuid: self.uuid.clone(),
            close_reason: self.close_reason.clone(),
            elapsed: self.start.elapsed(),
        });
        Running::Stop
    }
}

impl Handler<OnlineUserCountMessage> for PlaceSession {
    type Result = ();

    fn handle(&mut self, msg: OnlineUserCountMessage, ctx: &mut Self::Context) -> Self::Result {
        let mut session = self.session.clone();
        async move {
            session.text(format!("/count {}", msg.0)).await.ok();
        }
        .into_actor(self)
        .wait(ctx);
    }
}

impl Handler<model::PixelColorUpdateMessage> for PlaceSession {
    type Result = ();

    fn handle(&mut self, msg: model::PixelColorUpdateMessage, ctx: &mut Self::Context) -> Self::Result {
        let mut session = self.session.clone();
        async move {
            session.binary(msg.serialize()).await.ok();
        }
        .into_actor(self)
        .wait(ctx);
    }
}

impl Handler<WsMessage> for PlaceSession {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) -> Self::Result {
        let msg: ws::Message = msg.0;
        let mut session = self.session.clone();
        let place_server = self.place_server.clone();
        let uuid = self.uuid.clone();

        async move {
            let close_reason = match msg {
                ws::Message::Ping(msg) => { session.pong(&msg).await.ok(); None },
                ws::Message::Text(text) => { session.text(text).await.ok(); None },
                ws::Message::Binary(bin) => { place_server.send(UserPixelColorMessage::new(uuid, &bin).unwrap()).await.ok(); None },
                ws::Message::Close(reason) => {
                    Some(reason)
                },
                _ => { None },
            };
            close_reason
        }
        .into_actor(self) // converts future to ActorFuture
        .then(|res, act, ctx| {
            if let Some(opt_reason) = res {
                act.close(opt_reason, ctx);
            }
            fut::ready(())
        })
        .wait(ctx);
    }
}

impl Handler<StopSession> for PlaceSession {
    type Result = ();

    fn handle(&mut self, msg: StopSession, ctx: &mut Self::Context) -> Self::Result {
        self.close(msg.0, ctx);
    }
}
