use actix::{Actor, Addr};
use actix_web::{web, HttpRequest, HttpResponse};
use tokio::task::spawn_local;
use futures_util::stream::StreamExt as _; // get .next() working

use super::{messages::{StopSession, WsMessage}, PlaceServer, PlaceSession};

pub async fn handle_ws(
    uuid: String,
    req: HttpRequest,
    body: web::Payload,
    place_server: web::Data<Addr<PlaceServer>>,
) -> actix_web::Result<HttpResponse> {
    let (response, session, mut msg_stream) = actix_ws::handle(&req, body)?;

    spawn_local(async move {
        let place_session = PlaceSession::new(uuid.clone(), place_server.get_ref().clone(), session).start();

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
    });


    Ok(response)
}