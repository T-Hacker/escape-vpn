mod connection_info;
mod connection_status;

use connection_info::ConnectionInfo;
use connection_status::TcpConnectionStatus;

fn main() {
    let pid = 3189;
    let connections_pending = find_pending_connections_from_pid(pid);

    for connection in connections_pending {
        println!("{:?}", connection);

        break;
    }
}

fn find_pending_connections_from_pid(pid: isize) -> Vec<std::net::IpAddr> {
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
