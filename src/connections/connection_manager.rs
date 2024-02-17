use super::{Connection, ConnectionState};
use std::{
    io::Write,
    net::Ipv4Addr,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex, OnceLock},
    time::Instant,
};

static CONNECTION_MANAGER: OnceLock<Arc<Mutex<ConnectionManager>>> = OnceLock::new();

#[derive(Default)]
pub struct ConnectionManager {
    connections: Vec<Connection>,
}

impl ConnectionManager {
    pub fn add_connection(&mut self, connection: Connection) {
        // Save connections to file.
        let connection_file = get_connection_file_path();
        let mut file = match std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(connection_file)
        {
            Ok(file) => file,
            Err(e) => {
                log::error!("Fail to open connection file: {e}");

                return;
            }
        };

        writeln!(file, "{}", connection.address().to_string()).unwrap_or_default();

        // Register connection.
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
    CONNECTION_MANAGER
        .get_or_init(|| {
            let start_time = Instant::now();

            let connection_file = get_connection_file_path();
            let connections = std::fs::read_to_string(connection_file).unwrap_or_default();
            let connections = connections
                .split_whitespace()
                .filter_map(|line| {
                    let Ok(address) = Ipv4Addr::from_str(line) else {
                        return None;
                    };

                    Some(Connection::new(
                        address,
                        ConnectionState::Pending { start_time },
                    ))
                })
                .collect();

            Arc::new(Mutex::new(ConnectionManager { connections }))
        })
        .clone()
}

fn get_connection_file_path() -> PathBuf {
    let path = std::env::temp_dir()
        .join(env!("CARGO_PKG_NAME"))
        .join("connections.txt");

    std::fs::create_dir_all(&path).unwrap_or_default();

    path
}
