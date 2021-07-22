use actix::*;
use actix_cors::Cors;

use library::{
    db::Actions, rest_api::rest_config, templates, ws_route, RelayServer, SessionClient,
};
use std::sync::{atomic::AtomicUsize, Arc};

use actix_files as fs;
use actix_web::{http::header, middleware, web, App, HttpRequest, HttpServer, Responder};

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
    let server = RelayServer::new(app_state.clone(), db_actions.clone()).start();

    // initialize sqlite db if not already initialized

    SessionClient::spawn("127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            // configure CORS
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            // enable logger
            .wrap(middleware::Logger::default())
            // test route
            .route("/test", web::get().to(greet))
            .route("/test/{name}", web::get().to(greet))
            // relay_server
            .data(server.clone())
            .data(app_state.clone())
            // db actions
            .data(db_actions.clone())
            // websocket route
            .service(web::resource("/ws/").to(ws_route))
            // confiure REST api
            .configure(rest_config)
            // static files
            .service(fs::Files::new("/static", "./static").index_file("./static/404.html"))
            .service(web::resource("/").route(web::get().to(templates::index)))
    })
    // start http server on 127.0.0.1:8080
    .bind(ADDRESS)?
    .run()
    .await
}
