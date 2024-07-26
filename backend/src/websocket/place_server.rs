use std::collections::HashMap;

use actix::prelude::*;
use redis;

use super::place_session::PlaceSession;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Debug, Clone)]
pub struct PlaceServer {
    redis_client: redis::Client,
    sessions: HashMap<String, Addr<PlaceSession>>,
}

/// New session is created
#[derive(Message)]
#[rtype(String)]
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
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct OnlineUserCountMessage(pub usize);

impl PlaceServer {
    pub fn new(redis_client: redis::Client) -> Self {
        Self {
            redis_client,
            sessions: HashMap::new()
        }
    }
}

impl PlaceServer {
    fn send_message(&self, _message: OnlineUserCountMessage, _skip_uuid: Option<String>)
    {
        unimplemented!("to do")
    }
}

impl Actor for PlaceServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

impl Handler<ConnectMessage> for PlaceServer {
    type Result = String;

    fn handle(&mut self, msg: ConnectMessage, _: &mut Context<Self>) -> Self::Result {
        self.sessions.insert(msg.author_uuid.clone(), msg.addr);

        let count = self.sessions.len();
        let message_count = OnlineUserCountMessage(count);
        dbg!(&message_count);
        self.send_message(message_count, None);

        msg.author_uuid
    }
}

impl Handler<DisconnectMessage> for PlaceServer {
    type Result = ();

    fn handle(&mut self, msg: DisconnectMessage, _: &mut Context<Self>) -> Self::Result {
        self.sessions.remove(&msg.author_uuid);

        let count = self.sessions.len();
        let message_count = OnlineUserCountMessage(count);
        self.send_message(message_count, None);
    }
}
