use actix_web::{get, post, web, Error, HttpRequest, HttpResponse, Responder};
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use tokio::sync::mpsc::Sender;

use crate::{
    routes::symbols::retrieve_symbols,
    state::client::{StreamRequest, WsState},
    CLIENT_COUNTER,
};
pub mod symbols;
mod ws;
#[get("/ws")]
pub async fn ws_route(
    req: actix_web::HttpRequest,
    stream: web::Payload,
    dis_tx: web::Data<Sender<StreamRequest>>,
) -> Result<HttpResponse, Error> {
    CLIENT_COUNTER.fetch_add(1, Ordering::SeqCst);
    let client = WsState::new(CLIENT_COUNTER.load(Ordering::SeqCst));

    dbg!(&client);

    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;
    actix_web::rt::spawn(ws::ws_client(session, msg_stream, dis_tx.as_ref().clone()));

    Ok(res)
}

#[get("/")]
pub async fn index(_req: actix_web::HttpRequest) -> impl Responder {
    "Hello"
}

#[get("/symbols")]
pub async fn symbols_all() -> Result<web::Json<serde_json::Value>, Error> {
    let v = retrieve_symbols(None, None).await.unwrap().into();
    Ok(web::Json(v))
}

#[get("/symbols/{exchange}")]
pub async fn exchange_symbols(
    exchange: web::Path<String>,
    _client: web::Data<awc::Client>,
) -> web::Json<serde_json::Value> {
    // let map = get_all_symbols().await;
    let map = retrieve_symbols(None, None).await.unwrap();
    if let Some(value) = map.get(&exchange.into_inner()) {
        return web::Json(value.clone());
    }
    return web::Json(serde_json::json!({
        "error":
            format!(
                "Invalid or unsupported exchange name. Supported Exchanges: ",
                // map.keys().collect::<Vec<String>>()
            )
    }));
}
