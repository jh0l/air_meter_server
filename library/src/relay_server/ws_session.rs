use std::time::Instant;

use actix::prelude::*;

use actix_web::{error, web, Error, HttpRequest, HttpResponse};

use actix_web_actors::ws;

use crate::{
    relay_server,
    relay_server::{
        server::RelayServer, Join as SubJoin, PublisherMessage as PubMsg, Reading, Role,
    },
};

use crate::{CLIENT_TIMEOUT, HEARTBEAT_INTERVAL};
use serde::{Deserialize, Serialize};
use serde_json::from_slice;

// Commands

/// Subscribe to publisher
#[derive(Message, Debug, Deserialize, Serialize)]
#[rtype(result = "()")]
pub struct Join {
    /// id of publisher client wants to subscribe to
    pub pub_id: usize,
}

pub struct WsSession {
    /// hb increment
    hb: Instant,
    /// relay server
    server_addr: Addr<RelayServer>,
    ses_role: Role,
}

fn from_json<'a, T>(des: &'a str) -> Result<T, String>
where
    T: Deserialize<'a>,
{
    from_slice::<T>(des.as_bytes()).map_err(|err| (format!("{:?}", err)))
}

impl WsSession {
    // helper method that sends intermittent ping to client
    // also checks ws client heartbeat and terminates session on timeout
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client hearbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("[srv/s] {:?} TIMED OUT, DISCONNECTING", act.ses_role);

                // stop actor
                ctx.stop();

                // do not ping
                return;
            };
            ctx.ping(b"");
        });
    }

    // helper method handles ws messages from client, parses msg then forwards
    // to appropriate relay server handler
    fn parse_message(&self, text: &str, _: &mut ws::WebsocketContext<Self>) -> Result<(), String> {
        let m = text.trim();
        // parse command
        let v: Vec<&str> = m.splitn(2, ' ').collect();
        let cmd = v[0];
        let msg = v[1].to_owned();
        match self.ses_role {
            Role::Publisher(pub_id) => match cmd {
                "/reading" => {
                    let msg = from_json::<Reading>(&msg)?;
                    self.server_addr.do_send(PubMsg::<Reading> { msg, pub_id });
                    Ok(())
                }
                _ => Err(format!("unrecognised command {}", cmd)),
            },
            Role::Subscriber(ses_id) => match cmd {
                "/join" => {
                    let payload = from_json::<Join>(&msg)?;
                    let Join { pub_id } = payload;
                    self.server_addr.do_send(SubJoin { ses_id, pub_id });
                    Ok(())
                }
                _ => Err(format!("unrecognised command {}", cmd)),
            },
        }
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
                ses_role: self.ses_role,
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.ses_role = act.ses_role.replace(res),
                    // something wrong
                    Err(err) => {
                        println!("[srv/s] WS CONNECT ERROR: {:?}", err);
                        ctx.stop();
                    }
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        println!("[srv/s] {:?} WS SESSION STOPPING", self.ses_role);
        // notify relay server
        self.server_addr.do_send(relay_server::Disconnect {
            ses_id: self.ses_role.into(),
        });
        Running::Stop
    }
}

// Handles messages from Websocket client, forwarding text to helper method
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(err) => {
                println!("[srv/s] RECEIVED ERROR FROM WS CLIENT {:?}", err);
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        println!("[srv/s] {:?}: {:?}", self.ses_role, msg);
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.ping(&msg);
            }
            ws::Message::Pong(_) => self.hb = Instant::now(),
            ws::Message::Text(text) => {
                self.parse_message(&text, ctx).unwrap_or_else(|err| {
                    ctx.text(&format!("COMMAND ERROR: {:?}", err));
                });
            }
            ws::Message::Binary(_) => println!("[srv/s] Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => ctx.stop(),
            ws::Message::Nop => (),
        }
    }
}

pub async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<RelayServer>>,
) -> Result<HttpResponse, Error> {
    let role: Result<Role, String> = match req.headers().get("authorization") {
        Some(auth) => match auth.to_str() {
            Ok(ses_id_str) => match ses_id_str.parse::<usize>() {
                Ok(ses_id) => Ok(Role::Publisher(ses_id)),
                Err(err) => {
                    println!("[srv/s] {:?}", err);
                    Err(format!("couldn't parse {}", ses_id_str))
                }
            },
            Err(err) => {
                println!("[srv/s] {:?}", err);
                Err("couldn't convert auth header to string".to_owned())
            }
        },
        None => Ok(Role::Subscriber(0)),
    };
    match role {
        Ok(role) => ws::start(
            WsSession {
                hb: Instant::now(),
                ses_role: role,
                server_addr: srv.get_ref().clone(),
            },
            &req,
            stream,
        ),
        Err(msg) => Err(error::ErrorBadRequest(msg)),
    }
}
