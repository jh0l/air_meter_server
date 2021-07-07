#[macro_use]
extern crate diesel;

use std::time::Duration;

/// How often heartbeat pings are sent
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
/// How long before lack of client response causes a timeout
pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

pub mod sensor_client;
pub use sensor_client::SessionClient;

pub mod relay_server;
pub use relay_server::{server::RelayServer, ws_route};

pub mod db;

pub mod schema;
