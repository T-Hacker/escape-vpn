use std::{
    cmp::Ordering,
    net::Ipv4Addr,
    sync::{Arc, Mutex, OnceLock},
    time::Instant,
};

static CONNECTION_MANAGER: OnceLock<Arc<Mutex<ConnectionManager>>> = Default::default();

pub enum ConnectionState {
    Idle,
    Pending,
    InRoutingTable,
}

pub struct Connection {
    address: Ipv4Addr,
    creation_time: Instant,
    state: ConnectionState,
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

impl Eq for Connection {}

impl PartialOrd for Connection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.address.partial_cmp(&other.address)
    }
}

impl Ord for Connection {
    fn cmp(&self, other: &Self) -> Ordering {
        self.address.cmp(&other.address)
    }
}

#[derive(Default)]
pub struct ConnectionManager {
    connections: Vec<Connection>,
}

impl ConnectionManager {
    pub fn add_connection(&mut self, connection: Connection) {
        // TODO: Add route to the network routing table.
        let Err(index) = self.connections.binary_search(&connection) else {
            log::warn!(
                "Connection already present: {}",
                connection.address.to_string()
            );

            return;
        };

        self.connections.insert(index, connection);
    }

    pub fn remove_connection(&mut self, address: &Ipv4Addr) -> Option<Connection> {
        // TODO: Remove from network routing table.
        let Ok(index) = self
            .connections
            .binary_search_by(|connection| connection.address.cmp(address))
        else {
            return None;
        };

        Some(self.connections.remove(index))
    }

    pub fn get_connection(&self, address: &Ipv4Addr) -> Option<&Connection> {
        let Ok(index) = self
            .connections
            .binary_search_by(|connection| connection.address.cmp(address))
        else {
            return None;
        };

        self.connections.get(index)
    }
}

pub fn get_connection_mananger() -> Arc<Mutex<ConnectionManager>> {
    // TODO: Implement connection persistence.
    CONNECTION_MANAGER
        .get_or_init(|| Default::default())
        .clone()
}
