use crate::schema::readings;
use actix::prelude::Message;

#[derive(Insertable, Debug)]
#[table_name = "readings"]
pub struct NewReading {
    pub publisher_id: i64,
    pub eco2: i32,
    pub evtoc: i32,
    pub read_time: i64,
    pub start_time: i64,
    pub increment: String,
}

#[derive(Queryable, Debug, Clone)]
// #[table_name = "readings"]
pub struct DbReading {
    pub id: i32,
    pub publisher_id: i64,
    pub eco2: i32,
    pub evtoc: i32,
    pub read_time: i64,
    pub start_time: i64,
    pub increment: String,
}

#[derive(Clone, Debug)]
pub struct GetReadings {
    pub pub_id: u64,
    pub before: Option<u64>,
    pub limit: i64,
}

impl Message for GetReadings {
    type Result = Vec<DbReading>;
}
