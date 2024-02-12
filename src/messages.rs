use color_eyre::eyre::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Message {
    AttachRequest { pid: u32, delay: u32 },
    AttachResponse { error: AttachError },

    DetachRequest { pid: u32 },
    DetachResponse { error: DetachError },
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum AttachError {
    Ok,
    ProcessNotFound,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum DetachError {
    Ok,
    ProcessNotFound,
}

pub fn serialize_to<T, W>(value: &T, writer: W) -> Result<()>
where
    T: serde::Serialize,
    W: std::io::Write,
{
    bincode::serialize_into(writer, value).wrap_err("Fail to serialize message.")
}

pub fn deserialize_from<T, R>(reader: R) -> Result<T>
where
    T: serde::de::DeserializeOwned,
    R: std::io::Read,
{
    bincode::deserialize_from::<_, T>(reader).wrap_err("Fail to deserialize message.")
}
