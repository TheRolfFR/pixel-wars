use std::pin::pin;

use actix::Addr;
use actix_web::{web, HttpRequest, HttpResponse};
use tokio::{sync::mpsc::unbounded_channel, task::spawn_local};
use futures_util::{
    future::{select, Either},
    StreamExt as _,
};

use super::{messages::{ConnectMessage, DisconnectMessage, ServerInternalMessage}, PlaceServer, PlaceSession};

pub async fn handle_ws(
    uuid: String,
    req: HttpRequest,
    body: web::Payload,
    place_server: web::Data<Addr<PlaceServer>>,
) -> actix_web::Result<HttpResponse> {
    let (response, session, msg_stream) = actix_ws::handle(&req, body)?;

    let (conn_tx, mut conn_rx) = unbounded_channel::<ServerInternalMessage>();

    spawn_local(async move {
        log::info!("Starting PlaceSession for #{}", uuid);
        place_server.send(ConnectMessage {
            author_uuid: uuid.clone(),
            session_tx: conn_tx
        }).await.ok();

        let place_session = PlaceSession::new(uuid.clone(), place_server.get_ref().clone());
        let mut msg_stream = pin!(msg_stream);
        let close_reason = loop {
            let msg_rx = pin!(conn_rx.recv());

            match select(msg_stream.next(), msg_rx).await {
                // WS message ok
                Either::Left((Some(Ok(ws_msg)),_)) => {
                    if let Some(reason) = place_session.handle_ws(ws_msg, session.clone()).await {
                        break reason; // received close message
                    };
                },
                // WS message error
                Either::Left((Some(Err(err)),_)) => {
                    log::error!("{}", err);
                    break None;
                },
                // WS stream ended
                Either::Left((None, _)) => {
                    break None;
                },
                // internal message ok
                Either::Right((Some(int_message),_)) => {
                    place_session.handle_message(int_message, session.clone()).await;
                },
                // internal message none
                Either::Right((None, _)) => {
                    unreachable!(
                        "all connection message senders were dropped; chat server may have panicked"
                    );
                }
            };
        };

        place_server.send(DisconnectMessage {
            author_uuid: uuid.clone(),
        }).await.ok();

        let _ = session.close(close_reason).await;

        log::info!("Stopping PlaceSession for #{}", uuid);
    });


    Ok(response)
}