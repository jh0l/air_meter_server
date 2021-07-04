//! Simple websocket client.
use std::env;
use std::{io, thread};

use actix::io::SinkWrite;
use actix::*;
use actix_codec::Framed;
use awc::{
    error::WsProtocolError,
    ws::{Codec, Frame, Message},
    BoxedSocket, Client,
};
use bytes::Bytes;
use futures::stream::{SplitSink, StreamExt};

use library::HEARTBEAT_INTERVAL;

fn main() {
    env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let sys = System::new("websocket-client");

    Arbiter::spawn(async {
        let backup = "http://192.168.0.67:8080/ws/".to_owned();
        let args: Vec<String> = env::args().collect();
        let address = args.get(1).unwrap_or(&backup);
        println!("Connecting to {:?}", address);
        let (response, framed) = Client::new()
            .ws(address)
            .connect()
            .await
            .map_err(|e| {
                println!("Error: {}", e);
            })
            .unwrap();

        println!("{:?}", response);
        let (sink, stream) = framed.split();
        let addr = ChatClient::create(|ctx| {
            ChatClient::add_stream(stream, ctx);
            ChatClient {
                sink: SinkWrite::new(sink, ctx),
                cache: 0,
            }
        });

        // start console loop
        thread::spawn(move || loop {
            let mut cmd = String::new();
            if io::stdin().read_line(&mut cmd).is_err() {
                println!("error");
                return;
            }
            addr.do_send(ClientCommand(cmd));
        });
    });
    sys.run().unwrap();
}

struct ChatClient {
    sink: SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>,
    cache: u64,
}

#[derive(Message)]
#[rtype(result = "()")]
struct ClientCommand(String);

#[derive(Message, Debug)]
#[rtype(result = "()")]
struct Heartbeat {
    cache: u64,
}

impl Actor for ChatClient {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        self.hb(ctx)
    }

    fn stopped(&mut self, _: &mut Context<Self>) {
        println!("Disconnected");

        // Stop application on disconnect
        System::current().stop();
    }
}

impl ChatClient {
    fn hb(&mut self, ctx: &mut Context<Self>) {
        ctx.notify(Heartbeat { cache: self.cache });
        // ctx.run_later(HEARTBEAT_INTERVAL, |act, ctx| {
        // self.sink.write(Message::Ping(Bytes::from_static(b"")));
        // self.hb(ctx);

        //     // client should also check for a timeout here, similar to the
        //     // server code
        // });
    }
}

/// Handle stdin commands
impl Handler<ClientCommand> for ChatClient {
    type Result = ();

    fn handle(&mut self, msg: ClientCommand, _: &mut Context<Self>) {
        let v: Vec<&str> = msg.0.trim().splitn(2, ' ').collect();
        let res = match v[0] {
            "/join" => Some(format!("/join {{ \"pub_id\": {} }}", v[1])),
            _ => {
                println!("Unknown command {}", msg.0);
                None
            }
        };
        if let Some(res) = res {
            self.sink.write(Message::Text(res));
        }
    }
}

/// Handle heartbeat intervals
impl Handler<Heartbeat> for ChatClient {
    type Result = ();

    fn handle(&mut self, msg: Heartbeat, ctx: &mut Context<Self>) {
        println!("{:?}", msg);
        self.sink.write(Message::Ping(Bytes::from_static(b"")));
        ctx.notify_later(msg, HEARTBEAT_INTERVAL);
    }
}

/// Handle server websocket messages
impl StreamHandler<Result<Frame, WsProtocolError>> for ChatClient {
    fn handle(&mut self, msg: Result<Frame, WsProtocolError>, _: &mut Context<Self>) {
        if let Ok(Frame::Text(txt)) = msg {
            println!("Server: {:?}", txt)
        }
    }

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("Connected");
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        println!("Server disconnected");
        ctx.stop()
    }
}

impl actix::io::WriteHandler<WsProtocolError> for ChatClient {}
