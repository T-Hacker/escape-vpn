mod connection_info;
mod connection_status;

use connection_info::ConnectionInfo;
use connection_status::TcpConnectionStatus;
use std::net::IpAddr;

fn main() {
    let pid = std::env::args().skip(1).next().unwrap();
    let pid = isize::from_str_radix(&pid, 10).unwrap();
    let connections_pending = find_pending_connections_from_pid(pid);

    for connection in connections_pending {
        println!("{:?}", connection);

        add_ip_to_routing_table(&connection);
    }
}

fn find_pending_connections_from_pid(pid: isize) -> Vec<IpAddr> {
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

fn add_ip_to_routing_table(ip: &IpAddr) {
    std::process::Command::new("ip")
        .args(["route", "add", &format!("{}/32", ip), "via", "192.168.1.1"])
        .spawn()
        .unwrap();
}
