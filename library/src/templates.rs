use crate::db::actions::{Actions, GetReadings};
use actix::prelude::*;
use actix_web::{web, HttpResponse, Result};
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct Index<'a> {
    template_readout: &'a str,
}

pub async fn index(actions: web::Data<Addr<Actions>>) -> Result<HttpResponse> {
    // GET ACCESS TO db/ACTIONS ACTOR FROM ACTIX DATA SERVICE
    // INSERT DATA INTO TEMPLATE
    let readings = actions
        .get_ref()
        .send(GetReadings { limit: 1 })
        .await
        .unwrap();
    let s = Index {
        template_readout: &format!("{:?}", readings),
    }
    .render()
    .unwrap();

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
