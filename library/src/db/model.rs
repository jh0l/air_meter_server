use crate::schema::readings;
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

#[derive(Queryable, Debug)]
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
