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

#[derive(Copy, Clone, Debug)]
pub enum Role {
    Publisher(usize),
    Subscriber(usize),
}

impl Into<usize> for Role {
    fn into(self) -> usize {
        match self {
            Role::Publisher(id) => id,
            Role::Subscriber(id) => id,
        }
    }
}
impl Role {
    // replace the id value property of the enum while persisting enum value
    pub fn replace(self, id: usize) -> Role {
        match self {
            Role::Subscriber(_) => Role::Subscriber(id),
            Role::Publisher(_) => Role::Publisher(id),
        }
    }
}

// client events for relay server communications

/// server sends this message to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

/// New client session with relay server is created
#[derive(Message, Clone, Debug)]
#[rtype(usize)]
pub struct Connect {
    pub ses_role: Role,
    pub addr: Recipient<Message>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub ses_id: usize,
}

/// Send message to specific subscription (used by publisher)
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct PublisherMessage {
    /// Peer message
    pub msg: String,
    /// publisher id
    pub pub_id: usize,
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

    fn message_session(&self, session_id: &usize, message: &str) {
        if let Some(addr) = self.sessions.get(session_id) {
            do_send_log(addr, message);
        }
        println!("error: session {} doesnt exist", session_id);
    }

    // Assign subscription entry to incoming address through publisher id
    // Create subscription entry if None
    fn connect_publisher(&mut self, msg: Connect) -> usize {
        let Connect { ses_role, .. } = msg;
        println!("{:?} PUBLISHER CONNECTED", ses_role);

        // remove existing address if some exists
        if let Some(addr) = self.sessions.get(&ses_role.into()) {
            do_send_log(addr, "disconnected");
            self.sessions.remove(&ses_role.into());
        }

        // create subscription entry if none
        if self.subs.get(&ses_role.into()).is_none() {
            self.subs.insert(ses_role.into(), HashSet::new());
        };
        ses_role.into()
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
        println!("{:?}", msg);

        self.visitor_count.fetch_add(1, Ordering::SeqCst);

        let id: usize = match msg.ses_role {
            Role::Publisher(_) => {
                self.connect_publisher(msg.clone());
                msg.ses_role.into()
            }
            _ => self.rng.gen::<usize>(),
        };
        self.sessions.insert(id, msg.addr);
        id
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
        if let Some(sessions) = self.subs.get(&msg.pub_id) {
            println!("{:?}", msg);
            for user_id in sessions {
                self.message_session(user_id, msg.msg.as_str());
            }
        } else {
            println!("UNKNOWN PUBLISHER {}", msg.pub_id);
        }
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
