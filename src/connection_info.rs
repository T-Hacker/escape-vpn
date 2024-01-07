use crate::connection_status::TcpConnectionStatus;
use std::net::{AddrParseError, Ipv4Addr};

#[derive(Debug)]
pub struct ConnectionInfo {
    pub remote_address: Ipv4Addr,
    pub status: TcpConnectionStatus,
}

#[derive(Debug)]
pub struct ParseConnectionInfoError;

impl TryFrom<&str> for ConnectionInfo {
    type Error = ParseConnectionInfoError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut columns = value.split_whitespace().skip(2); // Skip index and local address.

        let column = columns.next().ok_or_else(|| ParseConnectionInfoError)?;
        let remote_address = parse_ipv4(column).map_err(|_| ParseConnectionInfoError)?;

        let column = columns.next().ok_or_else(|| ParseConnectionInfoError)?;
        let status = u8::from_str_radix(column, 16)
            .map_err(|_| ParseConnectionInfoError)?
            .try_into()
            .map_err(|_| ParseConnectionInfoError)?;

        Ok(ConnectionInfo {
            remote_address,
            status,
        })
    }
}

fn parse_ipv4(text: &str) -> Result<Ipv4Addr, AddrParseError> {
    let text = format!(
        "{}.{}.{}.{}",
        u8::from_str_radix(&text[0..2], 16).unwrap(),
        u8::from_str_radix(&text[2..4], 16).unwrap(),
        u8::from_str_radix(&text[4..6], 16).unwrap(),
        u8::from_str_radix(&text[6..8], 16).unwrap()
    );

    text.parse::<Ipv4Addr>()
}
