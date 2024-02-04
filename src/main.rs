mod client;
mod connection_info;
mod connection_status;
mod service;

use clap::{Parser, Subcommand};
use client::{attach, launch};
use connection_info::ConnectionInfo;
use connection_status::TcpConnectionStatus;
use service::service;
use std::{
    net::Ipv4Addr,
    process::Command,
    time::{Duration, Instant},
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
        pid: u32,
    },

    #[command(about = "Launch application as a service")]
    Service {
        #[arg(
            short,
            long,
            default_value_t = 1000,
            help = "Number of milisenconds between connection check."
        )]
        pooling_rate: u64,

        #[arg(
            short,
            long,
            default_value_t = 30000,
            help = "Number of milisenconds that a connection must be waiting before is added to the routing table."
        )]
        delay: u64,
    },
}

struct Connection {
    pub address: Ipv4Addr,
    pub creation_time: Instant,
    pub is_in_routing_table: bool,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Launch { command } => launch(&command),
        Commands::Attach { pid } => attach(pid),
        Commands::Service {
            pooling_rate,
            delay,
        } => {
            let pooling_rate = Duration::from_millis(pooling_rate);
            let delay = Duration::from_millis(delay);

            service(pooling_rate, delay);
        }
    }
}

fn find_ipv4_pending_connections_from_pid(pid: u32) -> Vec<Ipv4Addr> {
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
