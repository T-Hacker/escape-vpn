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

const MAX_VISIBILITY_COUNT: u32 = 1;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(
        short,
        long,
        default_value_t = 1000,
        help = "Number of milisenconds between connection check"
    )]
    pooling_rate: u64,

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

struct Connection {
    pub address: Ipv4Addr,
    pub visibility_count: u32,
    pub is_in_routing_table: bool,
}

fn main() {
    let cli = Cli::parse();
    let pooling_rate = Duration::from_millis(cli.pooling_rate);

    let connections: Arc<Mutex<Vec<Connection>>> = Default::default();

    // Handle application termination.
    {
        let connections = connections.clone();
        ctrlc::set_handler(move || {
            println!("Shutting down...");

            let connections = connections.lock().unwrap();
            for connection in connections.iter() {
                if !connection.is_in_routing_table {
                    continue;
                }

                println!(
                    "Removing address {} from routing table...",
                    connection.address
                );

                remove_ip_from_routing_table(&connection.address);
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
            monitor_process(pid, connections, pooling_rate);
        }
    }
}

fn monitor_process(pid: isize, connections: Arc<Mutex<Vec<Connection>>>, pooling_rate: Duration) {
    loop {
        {
            let connections_pending = find_ipv4_pending_connections_from_pid(pid);

            let mut connections = connections
                .lock()
                .expect("Unable to lock for write remote connections");

            // Update count of known peding connections.
            for address in &connections_pending {
                if let Some(connection) = connections
                    .iter_mut()
                    .find(|connection| connection.address == *address)
                {
                    if !connection.is_in_routing_table {
                        connection.visibility_count += 1;

                        println!(
                            "Address {} visibility has been bumped to {}.",
                            connection.address, connection.visibility_count
                        );
                    }
                } else {
                    connections.push(Connection {
                        address: *address,
                        visibility_count: 0,
                        is_in_routing_table: false,
                    });
                }
            }

            // Find connections to add to the routing table.
            for connection in connections.iter_mut() {
                if connection.is_in_routing_table
                    || connection.visibility_count < MAX_VISIBILITY_COUNT
                {
                    continue;
                }

                println!("Adding {} to routing table...", connection.address);
                add_ip_to_routing_table(&connection.address);

                connection.is_in_routing_table = true;
            }
        }

        sleep(pooling_rate);
    }
}

fn find_ipv4_pending_connections_from_pid(pid: isize) -> Vec<Ipv4Addr> {
    let tcp_file = std::fs::read_to_string(format!("/proc/{}/net/tcp", pid)).unwrap();

    let mut addresses = tcp_file
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
        .collect::<Vec<_>>();

    addresses.dedup();

    addresses
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
