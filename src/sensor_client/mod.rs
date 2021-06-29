use std::thread;

use actix::io::SinkWrite;
use actix::*;
use actix_codec::Framed;
use awc::{
    error::WsProtocolError,
    ws::{Codec, Frame, Message},
    BoxedSocket, Client,
};
use futures::stream::{SplitSink, StreamExt};

use bytes::Bytes;

use library::HEARTBEAT_INTERVAL;

mod ccs811;
use ccs811::Sensor;

pub struct SensorClient {
    sink: SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>,
    sensor: Sensor,
}

#[derive(Message)]
#[rtype(result = "()")]
struct SensorReading(String);

impl Actor for SensorClient {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        self.hb(ctx)
    }

    fn stopped(&mut self, _: &mut Context<Self>) {
        println!("Disconnected");

        // Stop application on disconnect
        println!("STOPPING SENSOR CLIENT");
        System::current().stop();
    }
}

/// Handle stdin commands
impl Handler<SensorReading> for SensorClient {
    type Result = ();

    fn handle(&mut self, msg: SensorReading, _ctx: &mut Context<Self>) {
        self.sink.write(Message::Text(msg.0));
    }
}

/// Handle server websocket messages
impl StreamHandler<Result<Frame, WsProtocolError>> for SensorClient {
    fn handle(&mut self, msg: Result<Frame, WsProtocolError>, _: &mut Context<Self>) {
        if let Ok(Frame::Text(txt)) = msg {
            println!("Server: {:?}", txt)
        }
    }

    fn started(&mut self, _: &mut Context<Self>) {
        println!("Sensor Client Connected");
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        println!("Sensor Client Disconnected");
        ctx.stop()
    }
}

impl actix::io::WriteHandler<WsProtocolError> for SensorClient {}

impl SensorClient {
    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_later(HEARTBEAT_INTERVAL, |act, ctx| {
            act.sink.write(Message::Ping(Bytes::from_static(b"")));
            act.hb(ctx);

            // client should also check for a timeout here, similar to the
            // server code
        });
    }

    pub fn spawn(server_url: &'static str) {
        let server_url = &(*server_url);
        Arbiter::spawn(async move {
            let mut url = "http://".to_owned();
            url.push_str(server_url);
            url.push_str("/ws/");
            let (response, framed) = Client::new()
                .ws(url)
                .set_header("authorization", "811")
                .connect()
                .await
                .map_err(|e| {
                    println!("Error: {}", e);
                })
                .unwrap();
            println!("ws response {:?}", response);
            let (sink, stream) = framed.split();
            let addr = SensorClient::create(|ctx| {
                SensorClient::add_stream(stream, ctx);
                SensorClient {
                    sink: SinkWrite::new(sink, ctx),
                    sensor: Sensor::new_1s().unwrap(),
                }
            });

            // start sensor reading loop
            thread::spawn(move || {
                // initialize ccs811 sensor
                let mut sensor = Sensor::new_1s().unwrap();
                loop {
                    // assign blocking sensor reading
                    let read = sensor.read().unwrap();
                    let cmd = format!(
                        "{{ \"eco2\": {} \"evtoc\":{} \"increment\":{} \"read_time\":{} \"start_time\":{} }}",
                        read.eco2, read.evtoc, read.increment, read.read_time, read.start_time
                    );
                    addr.do_send(SensorReading(cmd));
                }
            });
        });
    }
}
