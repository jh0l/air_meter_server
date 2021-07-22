use crate::{
    common::GetReadings,
    db::{actions::Actions, model::DbReading},
};
use actix::prelude::*;
use actix_web::{web, Error, HttpResponse};

pub async fn get_readings(
    web::Query(query): web::Query<GetReadings>,
    actions: web::Data<Addr<Actions>>,
) -> Result<HttpResponse, Error> {
    let readings = actions.get_ref().send(query).await.unwrap();
    let res = readings.iter().rev().collect::<Vec<&DbReading>>();
    Ok(HttpResponse::Ok().json(res))
}
