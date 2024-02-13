use crate::{
    connections::{
        get_connection_mananger,
        tcp::{TcpConnectionInfo, TcpConnectionStatus},
        Connection, ConnectionState,
    },
    get_service_address_file,
    messages::{deserialize_from, serialize_to, AttachError, Message},
    process_manager::{add_process, remove_process_and_trigger_exit},
};
use color_eyre::eyre::Result;
use std::{
    net::{Ipv4Addr, TcpListener, TcpStream},
    process::Command,
    sync::mpsc::{channel, Receiver, TryRecvError},
    time::{Duration, Instant},
};

pub fn service(address: &str, pooling_rate: Duration) {
    // Register service port.
    let port_file_name = get_service_address_file();
    log::info!(
        "Registering service port in: {}",
        port_file_name.to_str().unwrap()
    );
    let port = address.split(':').skip(1).next().unwrap();
    std::fs::write(&port_file_name, port).expect("Fail to register service port.");

    // Start listening for client connections.
    log::info!("Starting service in address {address}...");
    let listener = TcpListener::bind(address).expect("Fail to listen in address: {address}");

    // Handle client requests.
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,

            Err(e) => {
                log::error!("Fail to accept client: {e}");

                continue;
            }
        };

        // Decode request message.
        match deserialize_from::<Message, _>(&stream) {
            Ok(Message::AttachRequest { pid, delay }) => attach(pid, delay, pooling_rate, stream),
            Ok(Message::DetachRequest { pid }) => detach(pid),

            Ok(_) => {
                log::error!("Invalid message received.");

                continue;
            }

            Err(e) => {
                log::error!("Fail to decode request: {e}");

                continue;
            }
        }
    }
}

fn attach(pid: u32, delay: u32, pooling_rate: Duration, stream: TcpStream) {
    log::info!("Attaching to PID: {} with delay of {} ms...", pid, delay);

    let (sender, receiver) = channel();

    let join_handle = std::thread::spawn(move || {
        let delay = Duration::from_millis(delay as u64);

        track_process(pid, delay, pooling_rate, stream, receiver);
    });

    if add_process(pid, join_handle, sender).is_err() {
        log::error!("Fail to register process.");
    }
}

fn detach(pid: u32) {
    log::info!("Detaching from PID: {pid}...");

    match remove_process_and_trigger_exit(pid) {
        Ok(true) => log::info!("Successfuly detach from process: {pid}"),
        Ok(false) => log::warn!("Fail to detach from process: {pid}"),

        Err(e) => log::error!("Fail to detach from process: {pid} with error: {e}"),
    }
}

fn track_process(
    pid: u32,
    delay: Duration,
    pooling_rate: Duration,
    stream: TcpStream,
    exit_receiver: Receiver<()>,
) {
    loop {
        // Check if we should start cleaning up.
        match exit_receiver.try_recv() {
            Ok(_) => break,

            Err(TryRecvError::Disconnected) => unreachable!(),
            Err(TryRecvError::Empty) => { /* Do nothing. */ }
        }

        {
            let connections_pending = match get_ipv4_pending_connections_from_pid(pid) {
                Ok(connections_pending) => {
                    log::info!("Successfuly attached to process: {pid}");
                    send_attach_response(AttachError::Ok, &stream);

                    connections_pending
                }
                Err(e) => {
                    log::error!("Unable to find pending connections: {e}");
                    send_attach_response(AttachError::ProcessNotFound, &stream);

                    return;
                }
            };

            {
                // Lock connection manager.
                let connection_manager = get_connection_mananger();
                let Ok(mut connection_manager) = connection_manager.lock() else {
                    log::error!("Fail to lock connection manager.");

                    return;
                };

                // Add new connections.
                for address in &connections_pending {
                    if connection_manager.get_connection_mut(address).is_none() {
                        connection_manager.add_connection(Connection::new(
                            *address,
                            ConnectionState::Pending {
                                start_time: Instant::now(),
                            },
                        ));
                    }
                }

                // Find connections to add to the routing table.
                for connection in connection_manager.iter_mut() {
                    match connection.state() {
                        ConnectionState::Pending { start_time } => {
                            let elapsed = start_time.elapsed();
                            if elapsed < delay {
                                continue;
                            }

                            add_ip_to_routing_table(connection.address());
                            log::info!("Address {} added to routing table.", connection.address());

                            connection.set_state(ConnectionState::InRoutingTable);
                        }
                        ConnectionState::InRoutingTable => { /* Do nothing. */ }
                    }
                }
            }
        }

        std::thread::sleep(pooling_rate);
    }
}

fn get_ipv4_pending_connections_from_pid(pid: u32) -> Result<Vec<Ipv4Addr>> {
    let connections = get_ipv4_connection_info_from_pid(pid)?;

    let connections = connections
        .into_iter()
        .filter_map(|connection| {
            if connection.status() == &TcpConnectionStatus::SynSent {
                Some(connection.remote_address().to_owned())
            } else {
                None
            }
        })
        .collect();

    Ok(connections)
}

fn get_ipv4_connection_info_from_pid(pid: u32) -> Result<Vec<TcpConnectionInfo>> {
    let tcp_file = std::fs::read_to_string(format!("/proc/{}/net/tcp", pid))?;

    let mut connections: Vec<TcpConnectionInfo> = tcp_file
        .lines()
        .skip(1)
        .map(|line| line.try_into().unwrap())
        .collect();

    connections.dedup();

    Ok(connections)
}

fn add_ip_to_routing_table(ip: &Ipv4Addr) {
    Command::new("ip")
        .args(["route", "add", &format!("{}/32", ip), "via", "192.168.1.1"])
        .spawn()
        .unwrap();
}

// fn remove_ip_from_routing_table(ip: &Ipv4Addr) {
//     let ip = ip.to_string();
//
//     Command::new("ip")
//         .args(["route", "del", &ip])
//         .spawn()
//         .expect("Fail to remove IP from routing table.");
// }

fn send_attach_response(error: AttachError, stream: &TcpStream) {
    match serialize_to(&Message::AttachResponse { error }, stream) {
        Ok(_) => { /* Do nothing. */ }
        Err(e) => {
            log::error!("Fail to send response to service: {e}");

            return;
        }
    }
}
