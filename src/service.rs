use crate::{
    connection_info::ConnectionInfo,
    connection_status::TcpConnectionStatus,
    get_service_address_file,
    messages::{deserialize_from, serialize_to, AttachError, Message},
    process_manager::{add_process, remove_process_and_trigger_exit},
};
use color_eyre::eyre::Result;
use std::{
    net::{Ipv4Addr, TcpListener, TcpStream},
    process::Command,
    sync::{
        mpsc::{channel, Receiver, TryRecvError},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

struct Connection {
    pub address: Ipv4Addr,
    pub creation_time: Instant,
    pub is_in_routing_table: bool,
}

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
    let connections = Arc::new(Mutex::new(Vec::new()));
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
            Ok(Message::AttachRequest { pid, delay }) => {
                attach(pid, delay, pooling_rate, connections.clone(), stream)
            }
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

fn attach(
    pid: u32,
    delay: u32,
    pooling_rate: Duration,
    connections: Arc<Mutex<Vec<Connection>>>,
    stream: TcpStream,
) {
    log::info!("Attaching to PID: {} with delay of {} ms...", pid, delay);

    let (sender, receiver) = channel();

    let connections = connections.clone();
    let join_handle = std::thread::spawn(move || {
        let delay = Duration::from_millis(delay as u64);

        track_process(pid, connections, delay, pooling_rate, stream, receiver);
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
    connections: Arc<Mutex<Vec<Connection>>>,
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
            let connections_pending = match find_ipv4_pending_connections_from_pid(pid) {
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

            // Remove connections that are no longer pending.
            let mut connections = connections.lock().expect("Fail to lock connections");
            connections.retain(|connection: &Connection| {
                connection.is_in_routing_table || connections_pending.contains(&connection.address)
            });

            // Add new connections.
            let current_time = Instant::now();
            for address in &connections_pending {
                if connections
                    .iter()
                    .find(|connection| connection.address == *address)
                    .is_none()
                {
                    connections.push(Connection {
                        address: *address,
                        creation_time: current_time,
                        is_in_routing_table: false,
                    });
                }
            }

            // Find connections to add to the routing table.
            for connection in connections.iter_mut() {
                if connection.is_in_routing_table {
                    continue;
                }

                let elapsed_time = connection.creation_time.elapsed();
                if elapsed_time < delay {
                    continue;
                }

                log::info!("Adding {} to routing table...", connection.address);
                add_ip_to_routing_table(&connection.address);

                connection.is_in_routing_table = true;
            }
        }

        std::thread::sleep(pooling_rate);
    }
}

fn find_ipv4_pending_connections_from_pid(pid: u32) -> Result<Vec<Ipv4Addr>> {
    let tcp_file = std::fs::read_to_string(format!("/proc/{}/net/tcp", pid))?;

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

    Ok(addresses)
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

fn send_attach_response(error: AttachError, stream: &TcpStream) {
    match serialize_to(&Message::AttachResponse { error }, stream) {
        Ok(_) => { /* Do nothing. */ }
        Err(e) => {
            log::error!("Fail to send response to service: {e}");

            return;
        }
    }
}
