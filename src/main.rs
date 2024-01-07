mod connection_info;
mod connection_status;

use clap::{Parser, Subcommand};
use connection_info::ConnectionInfo;
use connection_status::TcpConnectionStatus;
use std::{
    net::Ipv4Addr,
    process::Command,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Launch a process and attach to it")]
    Launch {
        #[arg(required = true, help = "Command to execute and track connections.")]
        command: Vec<String>,
    },

    #[command(about = "Attach to a running process")]
    Attach {
        #[arg(required = true, help = "PID of the process to attach to.")]
        pid: isize,
    },
}

fn main() {
    let cli = Cli::parse();

    let remote_addresses = Arc::new(Mutex::new(vec![]));

    // Handle application termination.
    {
        let remote_addresses = remote_addresses.clone();
        ctrlc::set_handler(move || {
            println!("Shutting down...");

            let remote_addresses = remote_addresses.lock().unwrap();
            for address in remote_addresses.iter() {
                println!("Removing address {} from routing table...", address);

                remove_ip_from_routing_table(address);
            }

            std::process::exit(0);
        })
        .unwrap();
    }

    match cli.command {
        Commands::Launch { command } => {
            println!("Launching command: {:?}", command);

            todo!()
        }
        Commands::Attach { pid } => {
            monitor_process(pid, remote_addresses);
        }
    }
}

fn monitor_process(pid: isize, remote_addresses: Arc<Mutex<Vec<Ipv4Addr>>>) {
    loop {
        let connections_pending = {
            let mut connections_pending = find_ipv4_pending_connections_from_pid(pid);

            let mut remote_addresses = remote_addresses
                .lock()
                .expect("Unable to lock for write remote addresses");

            // Filter out addresses already present in the routing table.
            connections_pending.retain(|address| !remote_addresses.contains(address));

            // Store the remote addresses to later remove from routing table.
            for address in &connections_pending {
                remote_addresses.push(address.clone());
            }

            connections_pending
        };

        // Add all remote addresses to the routing table.
        for address in &connections_pending {
            println!("Adding address {:?} to routing table.", address);

            add_ip_to_routing_table(address);
        }

        sleep(Duration::from_secs(1));
    }
}

fn find_ipv4_pending_connections_from_pid(pid: isize) -> Vec<Ipv4Addr> {
    let tcp_file = std::fs::read_to_string(format!("/proc/{}/net/tcp", pid)).unwrap();

    tcp_file
        .lines()
        .skip(1)
        .filter_map(|line| {
            let connection_info: ConnectionInfo = line.try_into().unwrap();
            if connection_info.status == TcpConnectionStatus::SynSent {
                Some(connection_info.remote_address.into())
            } else {
                None
            }
        })
        .collect()
}

fn add_ip_to_routing_table(ip: &Ipv4Addr) {
    Command::new("ip")
        .args(["route", "add", &format!("{}/32", ip), "via", "192.168.1.1"])
        .spawn()
        .unwrap();
}

fn remove_ip_from_routing_table(ip: &Ipv4Addr) {
    let ip = ip.to_string();

    Command::new("ip")
        .args(["route", "del", &ip])
        .spawn()
        .expect("Fail to remove IP from routing table.");
}
