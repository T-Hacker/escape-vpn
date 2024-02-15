use super::TcpConnectionStatus;
use std::net::{AddrParseError, Ipv4Addr};

#[derive(Debug)]
pub struct TcpConnectionInfo {
    remote_address: Ipv4Addr,
    status: TcpConnectionStatus,
}

impl TcpConnectionInfo {
    pub fn remote_address(&self) -> &Ipv4Addr {
        &self.remote_address
    }

    pub fn status(&self) -> &TcpConnectionStatus {
        &self.status
    }
}

impl TryFrom<&str> for TcpConnectionInfo {
    type Error = ParseConnectionInfoError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut columns = value.split_whitespace().skip(2); // Skip index and local address.

        let column = columns.next().ok_or(ParseConnectionInfoError)?;
        let remote_address = parse_ipv4(column).map_err(|_| ParseConnectionInfoError)?;

        let column = columns.next().ok_or(ParseConnectionInfoError)?;
        let status = u8::from_str_radix(column, 16)
            .map_err(|_| ParseConnectionInfoError)?
            .try_into()
            .map_err(|_| ParseConnectionInfoError)?;

        Ok(TcpConnectionInfo {
            remote_address,
            status,
        })
    }
}

impl PartialEq for TcpConnectionInfo {
    fn eq(&self, other: &TcpConnectionInfo) -> bool {
        self.remote_address == other.remote_address && self.status == other.status
    }
}

fn parse_ipv4(text: &str) -> Result<Ipv4Addr, AddrParseError> {
    let text = format!(
        "{}.{}.{}.{}",
        u8::from_str_radix(&text[6..8], 16).unwrap(),
        u8::from_str_radix(&text[4..6], 16).unwrap(),
        u8::from_str_radix(&text[2..4], 16).unwrap(),
        u8::from_str_radix(&text[0..2], 16).unwrap(),
    );

    text.parse::<Ipv4Addr>()
}

#[derive(Debug)]
pub struct ParseConnectionInfoError;
