use std::sync::{
    atomic::{AtomicUsize},
    Arc,
};
use actix::*;

use actix_files as fs;
use actix_web::{middleware, web, App, HttpServer};

mod sensor_client;

use sensor_client::SensorClient;

mod relay_server;

use relay_server::RelayServer;

mod ws_session;

use ws_session::ws_route;

const ADDRESS: &str = "127.0.0.1:8080";


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    // App State
    // keep count of visitors
    let app_state = Arc::new(AtomicUsize::new(0));
    // TODO track server uptime

    //start relay server actor
    let server = RelayServer::new(app_state.clone()).start();

    // initialize sqlite db if not already initialized

    SensorClient::spawn(ADDRESS);

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            // relay_server
            .data(server.clone())
            .data(app_state.clone())
            // websocket route
            .service(web::resource("/ws/").to(ws_route))
            // static files
            .service(fs::Files::new("/", "static/").index_file("index.html"))
    })
    // start http server on 127.0.0.1:8080
    .bind(ADDRESS)?
    .run()
    .await
}
