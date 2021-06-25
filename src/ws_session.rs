use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use actix::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};

use actix_web_actors::ws;

use crate::relay_server;

pub struct WsSession {
    // session id
    ses_id: usize,
    // hb increment
    hb: Instant,
    // subscription id,
    sub_id: usize,
    // relay server
    server_addr: Addr<relay_server::RelayServer>,
}

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);


impl WsSession {
    // helper method that sends intermittent ping to client
    // also checks ws client heartbeat and terminates session on timeout
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client hearbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("{} TIMED OUT, DISCONNECTING", act.ses_id);

                // notify chat server
                act.server_addr.do_send(relay_server::Disconnect { ses_id: act.ses_id});

                // stop actor
                ctx.stop();

                // do not ping
                return
            };
            ctx.ping(b"");
        });
    }

    // helper method handles ws messages from client, parses msg then forwards
    // to appropriate relay server handler
    fn handle_client_message(&self, text: &str, ctx: &mut ws::WebsocketContext<Self>) {
        let m = text.trim();
        // check for publisher commands
    }
}


/// Handle messages from relay server, we simply send it to peer websocket
impl Handler<relay_server::Message> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: relay_server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    // Method is called on actor start
    // register ws session with RelayServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // start heartbeat with ws client
        self.hb(ctx);

        // TODO determine whether ws client is publisher or subscriber

        // register self in relay server. `AsyncContext::wait` register's
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsSession, state is shared
        // across all routes within application
        let addr = ctx.address();
        self.server_addr
            .send(relay_server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.ses_id = res,
                    // something wrong
                    Err(err) => {
                        println!("WS CONNECT ERROR: {:?}", err);
                        ctx.stop();
                    }
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        println!("{} WS SESSION STOPPING", self.ses_id);
        // notify relay server
        self.server_addr.do_send(relay_server::Disconnect { ses_id: self.ses_id });
        Running::Stop
    }
}

// Handles messages from Websocket client, forwarding text to helper method
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        let msg = match msg {
            Err(err) => {
                println!("RECEIVED ERROR FROM WS CLIENT {:?}", err);
                ctx.stop();
                return;
            },
            Ok(msg) => msg,
        };

        println!("WEBSOCKET MESSAGE: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.ping(&msg);
            }
            ws::Message::Pong(_) => self.hb = Instant::now(),
            ws::Message::Text(text) => self.handle_client_message(&text, ctx),
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => { ctx.close(reason); ctx.stop(); },
            ws::Message::Continuation(_) => ctx.stop(),
            ws::Message::Nop => ()
        }
    }
}

pub async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<relay_server::RelayServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        WsSession {
             ses_id: 0,
             hb: Instant::now(),
             sub_id: 0,
             server_addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}
