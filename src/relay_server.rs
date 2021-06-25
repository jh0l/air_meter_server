//! `RelayServer` is an actor. It maintains a map of client sessions.
//! Manages available subscriptions.
//! Publishing clients send messages to subscribed users through `RelayServer`.
//! Each publisher has its own subscription, multiple users can connect to a single
//! publisher's subscription

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use std::collections::{HashMap, HashSet};

// client events for relay server communications

/// server sends this message to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

/// New client session with relay server is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

/// New publisher session with relay server is created, assert subscription
#[derive(Message)]
#[rtype(result = "()")]
pub struct PublisherConnect {
    pub pub_id: usize,
    pub addr: Recipient<Message>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub ses_id: usize,
}

/// Send message to specific subscription (used by publisher)
#[derive(Message)]
#[rtype(result = "()")]
pub struct PublisherMessage {
    /// Peer message
    pub msg: String,
    /// subscription id
    pub sub_id: usize,
}

/// List of available subscriptions
pub struct ListSubs;

// list of publisher ids that can be subscribe to
impl actix::Message for ListSubs {
    type Result = Vec<usize>;
}

/// Join subscription, if non-existant throw error
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    /// session id
    pub ses_id: usize,
    /// Subscription id
    pub sub_id: usize,
}

// `RelayServer` manages 'subscriptions'
// relays publisher client readings to users
// assigns a subscriptions entry to each publisher client based on client ID
// subscriptions are a collectiono of users subscribed to publishers
// users are appended to subscriptions HashSet on joining
pub struct RelayServer {
    sessions: HashMap<usize, Recipient<Message>>,
    subs: HashMap<usize, HashSet<usize>>,
    rng: ThreadRng,
    visitor_count: Arc<AtomicUsize>,
}

impl RelayServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> RelayServer {
        // default subscription?
        let subs = HashMap::new();

        RelayServer {
            sessions: HashMap::new(),
            subs,
            rng: rand::thread_rng(),
            visitor_count
        }
    }

    fn message_session(&self, session_id: &usize, message: &str) -> Result<(), SendError<Message>> {
        if let Some(addr) = self.sessions.get(session_id) {
            return addr.do_send(Message(message.to_owned()));
        }
        Err(SendError::Closed(Message(message.to_owned())))
    }

    // send message to subscribers
    fn publish_message(&self, pub_id: &usize, message: &str) {
        if let Some(sessions) = self.subs.get(pub_id) {
            for user_id in sessions {
                self.message_session(user_id, message);
            }
        } else {
            println!("UNKNOWN PUBLISHER {}", pub_id);
        }
    }
}

// Make actor from `RelaySever`
impl Actor for RelayServer {
    // Simple context
    type Context = Context<Self>;
}

// Handler for Connect message
// Register a new session and assign unique id to this session
impl Handler<Connect> for RelayServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("{:?} CONNECTED", msg.addr);

        // register session with random id
        let id = self.rng.gen::<usize>();

        self.sessions.insert(id, msg.addr);

        self.visitor_count.fetch_add(1, Ordering::SeqCst);

        id
    }
}

// Handler for PublisherConnect message
// Assign incoming address to session of message's pub_id
// Create subscription entry if None
impl Handler<PublisherConnect> for RelayServer {
    type Result = ();

    fn handle(&mut self, msg: PublisherConnect, _: &mut Context<Self>) -> Self::Result {
        let PublisherConnect { addr, pub_id } = msg;
        println!("{} - {:?} PUBLISHER CONNECTED", pub_id, addr);

        // remove existing address if some exists
        if let Some(addr) = self.sessions.get(&pub_id) {
            addr.do_send(Message("disconnected".to_owned()));
            self.sessions.remove(&pub_id);
        }

        self.sessions.insert(pub_id, msg.addr);

        // create subscription entry if none
        if let None = self.subs.get(&pub_id) {
            self.subs.insert(pub_id, HashSet::new());
        };
    }
}

impl Handler<Disconnect> for RelayServer {
    type Result = ();
    
    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("ID: {} DISCONNECTING", msg.ses_id);

        // remove address
        if let Some(addr) = self.sessions.get(&msg.ses_id) {
            println!("ID: {} {:?} DISCONNECTED", msg.ses_id, addr);
            // remove session from all subscriptions
            for (_, sessions) in &mut self.subs {
                sessions.remove(&msg.ses_id);
            }
        }
    }
}

// Handler for Publisher message.
impl Handler<PublisherMessage> for RelayServer {
    type Result = ();

    fn handle(&mut self, msg: PublisherMessage, _: &mut Context<Self>) {
        self.publish_message(&msg.sub_id, msg.msg.as_str());
    }
}

// Handler for `List Publishers` message request.
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
        let Join {ses_id, sub_id} = msg;
        
        self.subs.get_mut(&sub_id)
        .and_then(|subs| {
            subs.insert(ses_id);
            Some(&sub_id)
        })
        .or_else(|| {
            self.message_session(&ses_id, format!("error:404 {} not found!", sub_id));
            None
        });
    }
}
