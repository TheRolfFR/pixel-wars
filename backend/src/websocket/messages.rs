use actix::prelude::*;
use tokio::sync::mpsc::UnboundedSender;

use crate::model::PixelColorUpdateMessage;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ConnectMessage {
    pub author_uuid: String,
    pub session_tx: UnboundedSender<ServerInternalMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DisconnectMessage {
    pub author_uuid: String,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct OnlineUserCountMessage(pub usize);

pub enum ServerInternalMessage {
    Pixel(PixelColorUpdateMessage),
    Online(usize)
}
