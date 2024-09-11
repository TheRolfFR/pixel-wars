use actix::{Actor, Addr};
use futures_util::stream::StreamExt as _; // get .next() working

use super::{messages::{StopSession, WsMessage}, PlaceServer, PlaceSession};

pub async fn handle_ws(
    uuid: String,
    session: actix_ws::Session,
    place_server: Addr<PlaceServer>,
    mut msg_stream: actix_ws::MessageStream,
) -> () {
    let place_session = PlaceSession::new(uuid, place_server, session).start();

    loop {
        match msg_stream.next().await {
            // WS message ok
            Some(Ok(ws_msg)) => {
                place_session.send(WsMessage(ws_msg)).await.ok();
            },
            Some(Err(err)) => {
                log::error!("{}",err);
                break;
            },
            None => {
                break;
            }
        };
    };

    place_session.do_send(StopSession(None));
}