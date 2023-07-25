use std::sync::atomic::AtomicUsize;

use actix_cors::Cors;
use actix_web::{http::header, middleware::Logger, web, App, HttpServer};
use tokio::sync;

use crate::state::client::StreamRequest;
mod routes;
mod state;

/// Global counter for all active clients
/// Increments by one when client connects
/// Decrements by one when client disconnects for any reason
///
/// Will also be used as the clients id when performing requests
static CLIENT_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> std::io::Result<()> {
    let (dis_tx, mut dis_rx) = sync::mpsc::channel::<StreamRequest>(500);

    tokio::spawn(async move {
        while let Some(cmd) = dis_rx.recv().await {
            dbg!(cmd);
        }
    });

    let port = std::env::var("PORT")
        .unwrap_or("5050".into())
        .parse::<u16>()
        .expect("Did you provide a proper positive integer?");

    #[cfg(not(debug_assertions))]
    log4rs::init_file("configs/log-config.yml", Default::default())
        .expect("Issue loading config file.\n looking for configs/log-config.yml");

    #[cfg(debug_assertions)]
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    log::info!("Starting HTTP server at http:/localhost:{port}");
    log::info!("GraphiQL playground: http://localhost:{port}/graphiql");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_methods(["GET", "POST"])
            .send_wildcard()
            .allow_any_origin()
            .allowed_headers(&[
                header::ACCEPT,
                header::UPGRADE,
                header::CONTENT_TYPE,
                header::SEC_WEBSOCKET_ACCEPT,
                header::SEC_WEBSOCKET_PROTOCOL,
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
            ])
            .max_age(3600);

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(awc::Client::default()))
            .app_data(web::Data::new(dis_tx.clone()))
            .service(routes::index)
            .service(routes::exchange_symbols)
            .service(routes::symbols_all)
            .service(routes::ws_route)
    })
    .bind(("0.0.0.0", port))?
    .workers(8)
    .run()
    .await
}
