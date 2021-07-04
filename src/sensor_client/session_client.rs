use actix::io::SinkWrite;
use actix::*;
use awc::{
    error::WsProtocolError,
    ws::{Frame, Message},
    Client,
};
use embedded_ccs811::MeasurementMode;
use futures::stream::StreamExt;
use std::time::Duration;

use bytes::Bytes;

use library::HEARTBEAT_INTERVAL;

use crate::sensor_client;
use crate::sensor_client::{ConnectSession, CurrentMode, Sensor, SessionClient, TakeReading};

#[derive(Message, Debug)]
#[rtype(result = "()")]
struct Heartbeat;

impl Actor for SessionClient {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        ctx.notify(Heartbeat);
        // give Sensor the sesion client's address
        self.sensor
            .try_send(ConnectSession {
                addr: ctx.address(),
            })
            .unwrap();
        println!("SESSION CLIENT STARTED");
    }

    fn stopped(&mut self, _: &mut Context<Self>) {
        println!("Disconnected");

        // Stop application on disconnect
        println!("STOPPING SENSOR CLIENT");
        System::current().stop();
    }
}

/// Handle messages (from Sensor actor) forward them to websocket session
impl Handler<sensor_client::Message> for SessionClient {
    type Result = ();

    fn handle(&mut self, msg: sensor_client::Message, _: &mut Context<Self>) {
        self.sink.write(Message::Text(msg.0));
    }
}

/// Handle receiving current mode from sensor - starts reading interval
impl Handler<CurrentMode> for SessionClient {
    type Result = ();
    fn handle(&mut self, msg: CurrentMode, ctx: &mut Context<Self>) {
        println!("sescli RECEIVED {:?}", msg);
        self.mode = Some(msg.inc);
        self.version += 1;
        ctx.notify(TakeReading {
            version: self.version,
        });
    }
}

/// implement sensor reading notify interval
impl Handler<TakeReading> for SessionClient {
    type Result = ();
    fn handle(&mut self, msg: TakeReading, ctx: &mut Context<Self>) {
        println!("sescli RECEIVED {:?} with self {:?}", msg, self.version);
        // check measurement mode hasn't been changed before reading
        if msg.version.eq(&self.version) {
            // sensor should have connected to session client in order for
            // CurrentMode to be received in order for TakeReading notification
            self.sensor.try_send(msg).unwrap();
            // self.mode should already be present in order for TakeReading notification
            let mode = self.mode_to_millis().unwrap();
            // queue reading for later
            ctx.notify_later(msg, Duration::from_millis(mode));
        } else {
            println!("{:?} does not match {:?}", self.version, msg.version);
        }
    }
}

/// Handle server websocket messages
impl StreamHandler<Result<Frame, WsProtocolError>> for SessionClient {
    fn handle(&mut self, msg: Result<Frame, WsProtocolError>, _: &mut Context<Self>) {
        if let Ok(Frame::Text(txt)) = msg {
            println!("Server: {:?}", txt)
        }
    }

    fn started(&mut self, _: &mut Context<Self>) {
        println!("Session Client Connected");
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        println!("Session Client Disconnected");
        ctx.stop()
    }
}

impl actix::io::WriteHandler<WsProtocolError> for SessionClient {}

/// Handle heartbeat intervals
impl Handler<Heartbeat> for SessionClient {
    type Result = ();

    fn handle(&mut self, msg: Heartbeat, ctx: &mut Context<Self>) {
        self.sink.write(Message::Ping(Bytes::from_static(b"")));
        ctx.notify_later(msg, HEARTBEAT_INTERVAL);
    }
}

impl SessionClient {
    fn mode_to_millis(&self) -> Option<u64> {
        let mut res = None;
        if let Some(mode) = self.mode {
            use MeasurementMode::*;
            let x = match mode {
                ConstantPower250ms => 250,
                ConstantPower1s => 1000,
                PulseHeating10s => 10000,
                LowPowerPulseHeating60s => 60000,
                Idle => u64::MAX,
            };
            res = Some(x);
        }
        res
    }

    pub fn spawn(server_url: &'static str) {
        let server_url = &(*server_url);
        Arbiter::spawn(async move {
            // thread spawn a ccs811 Sensor actor using SyncArbiter with access to session addr
            let sensor_add = SyncArbiter::start(1, || Sensor::new_1s().unwrap());
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
            SessionClient::create(|ctx| {
                SessionClient::add_stream(stream, ctx);
                SessionClient {
                    sink: SinkWrite::new(sink, ctx),
                    sensor: sensor_add.clone(), // initialize ccs811 sensor
                    mode: None,
                    version: 0,
                }
            });
        });
    }
}
