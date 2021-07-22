use actix::prelude::*;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::result::Error;

use crate::{
    common::GetReadings,
    db::model::{DbReading, NewReading},
    relay_server::{PublisherMessage as PubMsg, Reading},
};

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub struct Actions {
    pool: DbPool,
}

impl Actor for Actions {
    type Context = Context<Self>;
}

impl Handler<PubMsg<Reading>> for Actions {
    type Result = ();
    fn handle(&mut self, msg: PubMsg<Reading>, _: &mut Context<Self>) {
        let conn = self.conn();
        let rd = msg.msg;
        let new_reading = NewReading {
            publisher_id: msg.pub_id as i64,
            eco2: rd.eco2 as i32,
            evtoc: rd.evtoc as i32,
            read_time: rd.read_time as i64,
            start_time: rd.start_time as i64,
            increment: rd.increment,
        };
        let reading = conn
            .transaction::<_, Error, _>(|| {
                {
                    use crate::schema::readings;
                    diesel::insert_into(readings::table)
                        .values(&new_reading)
                        .execute(&conn)
                        .unwrap();
                }
                use crate::schema::readings::dsl::*;
                Ok(readings.order(id.desc()).first::<DbReading>(&conn).unwrap())
            })
            .map_err(|e| {
                println!("{:?}", e);
            });
        if let Err(reading) = reading {
            println!("FAILED TO INSERT NEW READING IN DB: {:?}", reading);
        }
    }
}

/// gets latest readings and returns them in ascending order by read_time
/// can select only latest readings before a certain read_time
impl Handler<GetReadings> for Actions {
    type Result = MessageResult<GetReadings>;

    fn handle(&mut self, msg: GetReadings, _: &mut Context<Self>) -> Self::Result {
        use crate::schema::readings::dsl::*;
        let query = readings.order(read_time.desc()).limit(msg.limit as i64);
        let result;
        if let Some(before) = msg.before {
            result = query
                .filter(read_time.lt(before as i64))
                .load::<DbReading>(&self.conn());
        } else {
            result = query.load::<DbReading>(&self.conn());
        }
        MessageResult(result.unwrap())
    }
}

embed_migrations!("../migrations");

impl Actions {
    pub fn new(connspec: &str) -> Actions {
        let manager = ConnectionManager::<SqliteConnection>::new(connspec);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");

        let conn = pool.get().unwrap();
        embedded_migrations::run(&conn).unwrap();
        Actions { pool }
    }

    fn conn(
        &mut self,
    ) -> r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::SqliteConnection>> {
        self.pool.get().unwrap()
    }
}
