use std::{cmp::Ordering, net::Ipv4Addr, time::Instant};

pub enum ConnectionState {
    Pending { start_time: Instant },
    InRoutingTable,
}

pub struct Connection {
    address: Ipv4Addr,
    state: ConnectionState,
}

impl Connection {
    pub fn new(address: Ipv4Addr, state: ConnectionState) -> Self {
        Self { address, state }
    }

    pub fn address(&self) -> &Ipv4Addr {
        &self.address
    }

    pub fn state(&self) -> &ConnectionState {
        &self.state
    }

    pub fn set_state(&mut self, value: ConnectionState) {
        self.state = value;
    }
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

impl Eq for Connection {}

impl PartialOrd for Connection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.address.cmp(&other.address))
    }
}

impl Ord for Connection {
    fn cmp(&self, other: &Self) -> Ordering {
        self.address.cmp(&other.address)
    }
}
