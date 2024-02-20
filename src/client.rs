use crate::{
    get_service_address_file,
    messages::{deserialize_from, serialize_to, AttachError, DetachError, Message},
};
use std::{io::Write, net::TcpStream, process::Command, time::Duration};

pub fn launch(command: &[String], delay: Duration) {
    let mut command = command.iter();
    let executable = command.next().expect("Executable name");

    let mut output = Command::new(executable)
        .args(command)
        .spawn()
        .expect("Fail to launch process");

    let pid = output.id();
    attach(pid, delay);

    output.wait().expect("Fail to wait for process");
}

pub fn attach(pid: u32, delay: Duration) {
    let mut stream = connect_to_service();

    // Send message to service.
    let msg = Message::AttachRequest {
        pid,
        delay: delay.as_millis() as u32,
    };
    serialize_to(&msg, &stream).expect("Fail to send message to service");
    stream.flush().expect("Fail to flush pipe");

    // Receive response from service.
    match deserialize_from::<Message, _>(stream).expect("Fail to read response message") {
        Message::AttachResponse { error } => match error {
            AttachError::Ok => println!("Successfuly attached to process: {pid}"),
            AttachError::ProcessNotFound => println!("Process not found."),
        },

        _ => panic!("Unexpected message received!"),
    }
}

pub fn detach_from_process(pid: u32) {
    let mut stream = connect_to_service();

    // Send message to service.
    let msg = Message::DetachRequest { pid };
    serialize_to(&msg, &stream).expect("Fail to send message to service");
    stream.flush().expect("Fail to flush pipe");

    // Receive response from service.
    let msg: Message = deserialize_from(stream).expect("Fail to read response message");
    match msg {
        Message::DetachResponse { error } => match error {
            DetachError::Ok => { /* Do nothing. */ }
            DetachError::ProcessNotFound => println!("Process not found!"),
            DetachError::UnknownError => println!("Unknown error occured in service!"),
        },

        _ => panic!("Unexpected message received!"),
    }
}

pub fn purge() {
    let mut stream = connect_to_service();

    // Send message to service.
    let msg = Message::PurgeRequest;
    serialize_to(&msg, &stream).expect("Fail to send message to service");
    stream.flush().expect("Fail to flush pipe");

    // Receive response from service.
    let msg: Message = deserialize_from(stream).expect("Fail to read response message");
    if msg != Message::PurgeResponse {
        println!("Fail to purge connections!");
    }
}

fn connect_to_service() -> TcpStream {
    // Find port of the service to connect to.
    let port_file_name = get_service_address_file();
    let port = std::fs::read_to_string(port_file_name).unwrap();
    let port = port.parse::<u16>().unwrap();
    let service_address = format!("127.0.0.1:{port}");

    // Connect to service.
    TcpStream::connect(service_address).unwrap()
}
