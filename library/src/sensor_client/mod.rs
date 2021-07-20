use embedded_ccs811::{mode, Ccs811Awake, MeasurementMode};
use linux_embedded_hal::I2cdev;

use actix::io::SinkWrite;

use actix_codec::Framed;
use awc::{
    ws::{Codec, Message as WsMessage},
    BoxedSocket,
};
use futures::stream::SplitSink;

use actix::prelude::{Addr, Message as ActixMessage};

mod ccs811;
mod session_client;

#[derive(ActixMessage, Debug)]
#[rtype(result = "()")]
pub struct Reading(pub String);

/// connect SessionClient and Sensor together
#[derive(ActixMessage, Clone)]
#[rtype(result = "()")]
pub struct ConnectSession {
    addr: Addr<SessionClient>,
}

/// Sensor tells SessionClient it's current MeasurementMode
#[derive(ActixMessage, Clone, Debug)]
#[rtype(result = "()")]
pub struct CurrentMode {
    inc: MeasurementMode,
}

/// tells the SessionClient to tell the Sensor to take a reading at intervals
#[derive(ActixMessage, Debug, Clone, Copy)]
#[rtype(result = "()")]
struct TakeReading {
    version: u64,
}

pub struct SessionClient {
    sink: SinkWrite<WsMessage, SplitSink<Framed<BoxedSocket, Codec>, WsMessage>>,
    sensor: Addr<Sensor>,
    mode: Option<MeasurementMode>,
    version: u64,
}

pub struct Sensor {
    pub_id: u64,
    app: Option<Ccs811Awake<I2cdev, mode::App>>,
    start_time: u64,
    increment: MeasurementMode,
    session: Option<Addr<SessionClient>>,
}
