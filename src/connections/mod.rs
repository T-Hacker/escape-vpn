mod connection;
mod connection_manager;

pub mod tcp;

pub use connection::{Connection, ConnectionState};
pub use connection_manager::get_connection_mananger;