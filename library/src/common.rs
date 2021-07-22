use crate::db::model::DbReading;
use actix::prelude::Message;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct GetReadings {
    pub pub_id: u64,
    pub before: Option<u64>,
    pub limit: u16,
}

impl Message for GetReadings {
    type Result = Vec<DbReading>;
}
