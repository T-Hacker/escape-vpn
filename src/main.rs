mod client;
mod connections;
mod messages;
mod monitoring;
mod process_manager;
mod service;

use clap::{Parser, Subcommand};
use client::{attach, detach_from_process, launch, purge};
use service::service;
use simple_logger::SimpleLogger;
use std::{path::PathBuf, time::Duration};

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

        #[arg(
            short,
            long,
            default_value_t = 30000,
            help = "Number of milisenconds that a connection must be waiting before is added to the routing table."
        )]
        delay: u32,
    },

    #[command(about = "Attach to a running process")]
    Attach {
        #[arg(required = true, help = "PID of the process to attach to.")]
        pid: u32,

        #[arg(
            short,
            long,
            default_value_t = 30000,
            help = "Number of milisenconds that a connection must be waiting before is added to the routing table."
        )]
        delay: u32,
    },

    #[command(about = "Detach to a running process")]
    Detach {
        #[arg(required = true, help = "PID of the process to detach from.")]
        pid: u32,
    },

    #[command(about = "Remove all connections from the routing table and caching.")]
    Purge,

    #[command(about = "Launch application as a service")]
    Service {
        #[arg(default_value = "127.0.0.1:3131", help = "Listening port.")]
        address: String,

        #[arg(
            short,
            long,
            default_value_t = 1000,
            help = "Number of milisenconds between connection check."
        )]
        pooling_rate: u32,
    },
}

fn main() {
    SimpleLogger::new()
        .init()
        .expect("Fail to initialize logger");

    let cli = Cli::parse();

    match cli.command {
        Commands::Launch { command, delay } => {
            let delay = Duration::from_millis(delay as u64);

            launch(&command, delay);
        }
        Commands::Attach { pid, delay } => {
            let delay = Duration::from_millis(delay as u64);

            attach(pid, delay);
        }
        Commands::Detach { pid } => detach_from_process(pid),
        Commands::Purge => purge(),

        Commands::Service {
            address,
            pooling_rate,
        } => {
            let pooling_rate = Duration::from_millis(pooling_rate as u64);

            service(&address, pooling_rate);
        }
    }
}

fn get_service_address_file() -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let exe_name = env!("CARGO_PKG_NAME");

    temp_dir.join(format!("{exe_name}.port"))
}
