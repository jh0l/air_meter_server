//! `RelayServer` is an actor. It maintains a map of client sessions.
//! Manages available subscriptions.
//! Publishing clients send messages to subscribed users through `RelayServer`.
//! Each publisher has its own subscription, multiple users can connect to a single
//! publisher's subscription
use crate::db::Actions;
use crate::relay_server::{
    Connect, Disconnect, Join, ListSubs, Message, PublisherMessage, Reading, Role,
};
use actix::prelude::*;
use rand::{rngs::ThreadRng, Rng};
use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

/// `RelayServer` manages 'subscriptions'
/// relays publisher client readings to users
/// assigns a subscriptions entry to each publisher client based on client ID
/// subscriptions are a collectiono of users subscribed to publishers
/// users are appended to subscriptions HashSet on joining
pub struct RelayServer {
    sessions: HashMap<u64, Recipient<Message>>,
    subs: HashMap<u64, HashSet<u64>>,
    rng: ThreadRng,
    visitor_count: Arc<AtomicUsize>,
    actions: Addr<Actions>,
}

fn do_send_log(addr: &actix::Recipient<Message>, message: &str) {
    if let Err(err) = addr.do_send(Message(message.to_owned())) {
        println!("[srv/m] do_send error: {:?}", err)
    }
}

/// Make actor from `RelaySever`
impl Actor for RelayServer {
    // Simple context
    type Context = Context<Self>;
}

impl RelayServer {
    pub fn new(visitor_count: Arc<AtomicUsize>, actions: Addr<Actions>) -> RelayServer {
        // default subscription?
        RelayServer {
            sessions: HashMap::new(),
            subs: HashMap::new(),
            rng: rand::thread_rng(),
            visitor_count,
            actions,
        }
    }

    fn message_session(&self, session_id: &u64, message: &str) {
        if let Some(addr) = self.sessions.get(session_id) {
            do_send_log(addr, message);
        } else {
            println!("[srv/m] error: session {} doesnt exist", session_id);
        }
    }

    // Assign subscription entry to incoming address through publisher id
    // Create subscription entry if None
    // Will override previously assigned address if existant
    fn connect_publisher(&mut self, ses_role: Role) -> u64 {
        // remove existing address if some exists
        if let Some(addr) = self.sessions.get(&ses_role.into()) {
            do_send_log(addr, "disconnected");
            self.sessions.remove(&ses_role.into());
        }

        // create subscription entry if none
        if self.subs.get(&ses_role.into()).is_none() {
            self.subs.insert(ses_role.into(), HashSet::new());
            println!("[srv/m] {:?} SUBSCRIPTION SET INIT'ED", ses_role);
        };
        println!("[srv/m] {:?} PUBLISHER CONNECTED", ses_role);
        ses_role.into()
    }
}

/// Handler for Connect message
/// Register a new session and assign unique id to this session
impl Handler<Connect> for RelayServer {
    type Result = u64;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("[srv/m] {:?}", msg);

        self.visitor_count.fetch_add(1, Ordering::SeqCst);

        // if publisher, id is specified by publisher, else gen new id
        let id: u64 = match msg.ses_role {
            Role::Publisher(_) => {
                self.connect_publisher(msg.ses_role);
                msg.ses_role.into()
            }
            _ => self.rng.gen::<u64>(),
        };
        self.sessions.insert(id, msg.addr);
        id
    }
}

impl Handler<Disconnect> for RelayServer {
    type Result = ();
    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("[srv/m] {:?}", msg);

        // remove address
        if self.sessions.get(&msg.ses_id).is_some() {
            println!("[srv/m] {:?} REMOVED", msg);
            // remove session from all subscriptions
            for sessions in &mut self.subs.values_mut() {
                sessions.remove(&msg.ses_id);
            }
        }
    }
}

/// Handler for Publisher message.
// impl Handler<PublisherMessage<String>> for RelayServer {
//     type Result = ();

//     fn handle(&mut self, msg: PublisherMessage<String>, _: &mut Context<Self>) {
//         if let Some(sessions) = self.subs.get(&msg.pub_id) {
//             println!("[srv/m] {:?}", msg);
//             for user_id in sessions {
//                 self.message_session(user_id, msg.msg.as_str());
//             }
//         } else {
//             println!("[srv/m] UNKNOWN PUBLISHER {}", msg.pub_id);
//         }
//     }
// }

/// Handler for Publisher message containing sensor reading
impl Handler<PublisherMessage<Reading>> for RelayServer {
    type Result = ();

    fn handle(&mut self, msg: PublisherMessage<Reading>, _: &mut Context<Self>) {
        if let Some(sessions) = self.subs.get(&msg.pub_id) {
            // send to db
            self.actions.do_send(msg.clone());
            // send to all subscribers
            for user_id in sessions {
                self.message_session(user_id, &format!("{:?}", msg.msg));
            }
        } else {
            println!("[srv/m] UNKNOWN PUBLISHER {}", msg.pub_id);
        }
    }
}

/// Handler for `List Publishers` message request.
impl Handler<ListSubs> for RelayServer {
    type Result = MessageResult<ListSubs>;

    fn handle(&mut self, _: ListSubs, _: &mut Context<Self>) -> Self::Result {
        let mut subs = Vec::new();

        for key in self.subs.keys() {
            subs.push(key.to_owned());
        }

        MessageResult(subs)
    }
}

impl Handler<Join> for RelayServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join { ses_id, pub_id } = msg;

        self.subs
            .get_mut(&pub_id)
            .map(|subs| if subs.insert(ses_id) { Some(()) } else { None })
            .map(|_| {
                self.message_session(&ses_id, &format!("joined {}", pub_id));
                Some(())
            })
            .or_else(|| {
                // TODO add reason for failure
                self.message_session(&ses_id, &format!("failed to join {}", pub_id));
                None
            });
    }
}
