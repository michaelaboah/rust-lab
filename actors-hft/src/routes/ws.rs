use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use actix_ws::{Closed, Message};
use futures_util::{
    future::{self, Either},
    StreamExt as _,
};
use tokio::{pin, sync::mpsc::Sender, time::interval};

use crate::{
    state::{
        client::{ClientEvent, ClientRequest, StreamRequest},
        server::ServerResponse,
    },
    CLIENT_COUNTER,
};

/// How often heartbeat pings are sent.
///
/// Should be half (or less) of the acceptable client timeout.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// How long before lack of client response causes a timeout.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

/// Echo text & binary messages received from the client, respond to ping messages, and monitor
/// connection health to detect network issues and free up resources.
pub async fn ws_client(
    mut session: actix_ws::Session,
    mut msg_stream: actix_ws::MessageStream,
    dispatch_tx: Sender<StreamRequest>,
) {
    log::info!("connected");
    let mut last_heartbeat = Instant::now();
    let mut interval = interval(HEARTBEAT_INTERVAL);

    let reason = loop {
        // create "next client timeout check" future
        let tick = interval.tick();
        // required for select()
        pin!(tick);

        // waits for either `msg_stream` to receive a message from the client or the heartbeat
        // interval timer to tick, yielding the value of whichever one is ready first
        match future::select(msg_stream.next(), tick).await {
            // received message from WebSocket client
            Either::Left((Some(Ok(msg)), _)) => {
                log::debug!("msg: {msg:?}");
                let session = &mut session;
                match msg {
                    Message::Text(text) => {
                        let response: ServerResponse =
                            match serde_json::from_str::<ClientEvent>(&text) {
                                Ok(event) => handle_message(event, session, &dispatch_tx).await,
                                Err(e) => handle_serde(&e, session).await,
                            };

                        let text = serde_json::to_string(&response).expect("No user input");

                        session.text(text).await.unwrap();
                    }

                    Message::Close(reason) => {
                        break reason;
                    }

                    Message::Ping(bytes) => {
                        last_heartbeat = Instant::now();
                        let _ = session.pong(&bytes).await;
                    }

                    Message::Pong(_) => {
                        last_heartbeat = Instant::now();
                    }

                    // Message::Continuation(_) => {
                    //     log::warn!("no support for continuation frames");
                    // }

                    // no-op; ignore
                    _ => {}
                };
            }

            // client WebSocket stream error
            Either::Left((Some(Err(err)), _)) => {
                log::error!("{}", err);
                break None;
            }

            // client WebSocket stream ended
            Either::Left((None, _)) => break None,

            // heartbeat interval ticked
            Either::Right((_inst, _)) => {
                // if no heartbeat ping/pong received recently, close the connection
                if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                    log::info!(
                        "client has not sent heartbeat in over {CLIENT_TIMEOUT:?}; disconnecting"
                    );

                    CLIENT_COUNTER.fetch_sub(1, Ordering::SeqCst);
                    break None;
                }

                // send heartbeat ping
                let _ = session.ping(b"").await;
            }
        }
    };

    // attempt to close connection gracefully
    let _ = session.close(reason).await;

    CLIENT_COUNTER.fetch_sub(1, Ordering::SeqCst);
    log::info!("disconnected");
}

async fn handle_serde(e: &serde_json::Error, session: &mut actix_ws::Session) -> ServerResponse {
    log::error!("Error parsing client json: {:?}", &e);

    let mut message = "".to_string();

    message = match e.classify() {
        serde_json::error::Category::Io => "Io error, failed to read or write bytes.".to_string(),
        serde_json::error::Category::Syntax => format!(
            "Syntax error on line, {} and column, {}",
            e.line(),
            e.column()
        ),
        serde_json::error::Category::Data => format!(
            "Type error, an unexpected type/kind of data was provided. Details, {}",
            e.to_string()
        ),
        serde_json::error::Category::Eof => "Empty messages are not valid inputs".to_string(),
    };

    ServerResponse::Error { message }
}

async fn handle_message(
    event: ClientEvent,
    session: &mut actix_ws::Session,
    dispatch_tx: &Sender<StreamRequest>,
) -> ServerResponse {
    match event {
        ClientEvent::Subscribe(s) => {
            if let Ok(req) = s.to_request() {
                dbg!(req);
                // dispatch_tx.send(req).await;
            }
        }
        ClientEvent::Unsubscribe(_) => todo!(),
        ClientEvent::Auth { key } => todo!(),
        ClientEvent::Status => todo!(),
    };

    ServerResponse::Info {
        message: "Filler".into(),
    }
}
