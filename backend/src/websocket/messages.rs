use std::time::Duration;

use actix::prelude::*;
use actix_ws as ws;

pub use crate::model::UserPixelColorMessage;

use super::PlaceSession;

#[derive(Message)]
#[rtype(result = "()")]
pub struct StopSession(pub Option<ws::CloseReason>);

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub ws::Message);

#[derive(Message)]
#[rtype(result = "()")]
pub struct ConnectMessage {
    pub uuid: String,
    pub addr: Addr<PlaceSession>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DisconnectMessage {
    pub uuid: String,
    pub close_reason: Option<ws::CloseReason>,
    pub elapsed: Duration,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct OnlineUserCountMessage(pub usize);
