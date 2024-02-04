use color_eyre::eyre::{Context, Result};
use nix::{sys::stat::Mode, unistd::mkfifo};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, Permissions},
    os::unix::fs::PermissionsExt,
    path::{self, Path},
    time::Duration,
};

#[derive(Serialize, Deserialize)]
pub struct AttachMessage {
    pid: u32,
}

impl AttachMessage {
    pub fn new(pid: u32) -> Self {
        Self { pid }
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn deserialize_from<R>(reader: R) -> Result<Self>
    where
        R: std::io::Read,
    {
        bincode::deserialize_from::<_, AttachMessage>(reader)
            .wrap_err("Fail to deserialize AttachMessage.")
    }
}

pub fn service(pooling_rate: Duration, delay: Duration) {
    println!("Starting service...");

    // Create named pipe for communication with clients.
    let path = Path::new("/var/run/escape-vpn");
    if !path.exists() {
        std::fs::create_dir(path).expect("Fail to create pipe application directory");
    }
    let path = path.join("escape-vpn-service.pipe");
    if path.exists() {
        std::fs::remove_file(&path)
            .expect("Fail to delete existing pipe. Maybe the service is already running?");
    }
    mkfifo(&path, Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IWGRP)
        .expect("Fail to create named pipe");
    std::os::unix::fs::chown(&path, None, Some(1002)).expect("Fail to change group of named pipe");
    std::fs::set_permissions(&path, Permissions::from_mode(0o660))
        .expect("Fail to change permissions of named pipe");

    // Open named pipe.
    let join_handle = std::thread::spawn(move || {
        println!("Waiting for clients...");

        loop {
            let pipe = std::fs::OpenOptions::new()
                .read(true)
                .open(&path)
                .expect("Fail to open named pipe");

            let Ok(msg) = bincode::deserialize_from::<_, AttachMessage>(&pipe) else {
                println!("Error reading request from client");

                continue;
            };

            println!("Attach to PID: {}", msg.pid());
        }
    });

    join_handle.join().expect("Fail to join pipe handle thread");

    println!("Done!");

    //
    // let connections: Arc<Mutex<Vec<Connection>>> = Default::default();
    //
    // // Handle application termination.
    // {
    //     let connections = connections.clone();
    //     ctrlc::set_handler(move || {
    //         println!("Shutting down...");
    //
    //         let connections = connections.lock().unwrap();
    //         for connection in connections.iter() {
    //             if !connection.is_in_routing_table {
    //                 continue;
    //             }
    //
    //             println!(
    //                 "Removing address {} from routing table...",
    //                 connection.address
    //             );
    //
    //             remove_ip_from_routing_table(&connection.address);
    //         }
    //
    //         std::process::exit(0);
    //     })
    //     .unwrap();
    // }
    //
    // loop {
    //     {
    //         let connections_pending = find_ipv4_pending_connections_from_pid(pid);
    //
    //         let mut connections = connections
    //             .lock()
    //             .expect("Unable to lock for write remote connections");
    //
    //         // Remove connections that are no longer pending.
    //         connections.retain(|connection| {
    //             connection.is_in_routing_table || connections_pending.contains(&connection.address)
    //         });
    //
    //         // Add new connections.
    //         let current_time = Instant::now();
    //         for address in &connections_pending {
    //             if connections
    //                 .iter()
    //                 .find(|connection| connection.address == *address)
    //                 .is_none()
    //             {
    //                 connections.push(Connection {
    //                     address: *address,
    //                     creation_time: current_time,
    //                     is_in_routing_table: false,
    //                 });
    //             }
    //         }
    //
    //         // Find connections to add to the routing table.
    //         for connection in connections.iter_mut() {
    //             if connection.is_in_routing_table {
    //                 continue;
    //             }
    //
    //             let elapsed_time = connection.creation_time.elapsed();
    //             if elapsed_time < delay {
    //                 continue;
    //             }
    //
    //             println!("Adding {} to routing table...", connection.address);
    //             add_ip_to_routing_table(&connection.address);
    //
    //             connection.is_in_routing_table = true;
    //         }
    //     }
    //
    //     sleep(pooling_rate);
    // }
}
