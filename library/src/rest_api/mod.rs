use actix_web::web;

pub mod handlers;
use handlers::sensors;

pub fn rest_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api").service(
            web::scope("/sensors").service(
                web::scope("/readings")
                    .service(web::resource("").route(web::get().to(sensors::get_readings))),
            ),
        ),
    );
}
