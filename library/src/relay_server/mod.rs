use actix::prelude::*;
use serde::Deserialize;

mod ws_session;

pub mod server;

pub use ws_session::ws_route;

#[derive(Copy, Clone, Debug)]
pub enum Role {
    Publisher(u64),
    Subscriber(u64),
}

#[allow(clippy::from_over_into)]
impl Into<u64> for Role {
    fn into(self) -> u64 {
        match self {
            Role::Publisher(id) => id,
            Role::Subscriber(id) => id,
        }
    }
}
impl Role {
    // replace the id value property of the enum while persisting enum value
    pub fn replace(self, id: u64) -> Role {
        match self {
            Role::Subscriber(_) => Role::Subscriber(id),
            Role::Publisher(_) => Role::Publisher(id),
        }
    }
}

// client events for relay server communications

/// server sends this message to session
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Message(pub String);

/// New client session with relay server is created
#[derive(Message, Clone, Debug)]
#[rtype(u64)]
pub struct Connect {
    pub ses_role: Role,
    pub addr: Recipient<Message>,
}

/// Session is disconnected
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub ses_id: u64,
}

/// Send message to publishers subscribers
#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct PublisherMessage<T>
where
    T: std::fmt::Debug,
{
    /// Peer message
    pub msg: T,
    /// publisher id
    pub pub_id: u64,
    // msg as original json string
    pub json: String,
}

/// Publisher reading
#[derive(Debug, Deserialize, Clone)]
pub struct Reading {
    pub pub_id: u64,
    pub eco2: u16,
    pub evtoc: u16,
    pub read_time: u64,
    pub start_time: u64,
    pub increment: String,
}

/// List of available subscriptions
pub struct ListSubs;

// list of publisher ids that can be subscribe to
impl actix::Message for ListSubs {
    type Result = Vec<u64>;
}

/// Join subscription, if non-existant throw error
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Join {
    /// session id of sender
    pub ses_id: u64,
    /// publisher id
    pub pub_id: u64,
}
