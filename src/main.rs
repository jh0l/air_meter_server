use library::{db::Actions, ws_route, RelayServer, SessionClient};

use actix::*;
use std::sync::{atomic::AtomicUsize, Arc};

use actix_files as fs;
use actix_web::{middleware, web, App, HttpRequest, HttpServer, Responder};

const ADDRESS: &str = "0.0.0.0:8080";
async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();
    dotenv::dotenv().ok();

    // App State
    // keep count of visitors
    let app_state = Arc::new(AtomicUsize::new(0));
    // TODO track server uptime

    // set up database connection pool
    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let db_actions = Actions::new(&connspec).start();

    //start relay server actor
    let server = RelayServer::new(app_state.clone(), db_actions).start();

    // initialize sqlite db if not already initialized

    SessionClient::spawn("127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            // test route
            .route("/test", web::get().to(greet))
            .route("/test/{name}", web::get().to(greet))
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
