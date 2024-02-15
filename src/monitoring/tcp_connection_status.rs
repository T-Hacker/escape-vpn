#[derive(Debug, PartialEq)]
pub enum TcpConnectionStatus {
    Established = 1,
    SynSent,
    SynRecv,
    FinWait1,
    FinWait2,
    TimeWait,
    Close,
    CloseWait,
    LastAck,
    Listen,
    Closing, /* Now a valid state */
    NewSynRecv,
}

#[derive(Debug)]
pub struct ParseConnectionStatusError;

impl TryFrom<u8> for TcpConnectionStatus {
    type Error = ParseConnectionStatusError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(TcpConnectionStatus::Established),
            2 => Ok(TcpConnectionStatus::SynSent),
            3 => Ok(TcpConnectionStatus::SynRecv),
            4 => Ok(TcpConnectionStatus::FinWait1),
            5 => Ok(TcpConnectionStatus::FinWait2),
            6 => Ok(TcpConnectionStatus::TimeWait),
            7 => Ok(TcpConnectionStatus::Close),
            8 => Ok(TcpConnectionStatus::CloseWait),
            9 => Ok(TcpConnectionStatus::LastAck),
            10 => Ok(TcpConnectionStatus::Listen),
            11 => Ok(TcpConnectionStatus::Closing),
            12 => Ok(TcpConnectionStatus::NewSynRecv),

            _ => Err(ParseConnectionStatusError),
        }
    }
}
