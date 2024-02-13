use super::Connection;
use std::{
    net::Ipv4Addr,
    sync::{Arc, Mutex, OnceLock},
};

static CONNECTION_MANAGER: OnceLock<Arc<Mutex<ConnectionManager>>> = OnceLock::new();

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
                connection.address().to_string()
            );

            return;
        };

        self.connections.insert(index, connection);
    }

    // pub fn remove_connection(&mut self, address: &Ipv4Addr) -> Option<Connection> {
    //     // TODO: Remove from network routing table.
    //     let Ok(index) = self
    //         .connections
    //         .binary_search_by(|connection| connection.address().cmp(address))
    //     else {
    //         return None;
    //     };
    //
    //     Some(self.connections.remove(index))
    // }
    //
    // pub fn get_connection(&self, address: &Ipv4Addr) -> Option<&Connection> {
    //     let Ok(index) = self
    //         .connections
    //         .binary_search_by(|connection| connection.address().cmp(address))
    //     else {
    //         return None;
    //     };
    //
    //     self.connections.get(index)
    // }

    pub fn get_connection_mut(&mut self, address: &Ipv4Addr) -> Option<&mut Connection> {
        let Ok(index) = self
            .connections
            .binary_search_by(|connection| connection.address().cmp(address))
        else {
            return None;
        };

        self.connections.get_mut(index)
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Connection> {
        self.connections.iter_mut()
    }
}

pub fn get_connection_mananger() -> Arc<Mutex<ConnectionManager>> {
    // TODO: Implement connection persistence.
    CONNECTION_MANAGER
        .get_or_init(|| Default::default())
        .clone()
}
