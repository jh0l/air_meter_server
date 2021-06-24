
use actix_files as fs;
use actix_web::{middleware, web, App, HttpServer};

mod sensor_websocket;

use sensor_websocket::ws_index;

mod sensor_client;

use sensor_client::SensorClient;

const ADDRESS: &str = "127.0.0.1:8080";


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    // initialize sqlite db if not already initialized

    SensorClient::spawn(ADDRESS);

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            // websocket route
            .service(web::resource("/ws/").route(web::get().to(ws_index)))
            // static files
            .service(fs::Files::new("/", "static/").index_file("index.html"))
    })
    // start http server on 127.0.0.1:8080
    .bind(ADDRESS)?
    .run()
    .await
}
