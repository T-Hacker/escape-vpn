use crate::service;
use std::{fs::OpenOptions, io::Write, process::Command};

pub fn launch(command: &[String]) {
    let mut command = command.iter();
    let executable = command.next().expect("Executable name");

    let output = Command::new(executable)
        .args(command)
        .spawn()
        .expect("Fail to launch process");

    let pid = output.id();

    attach(pid);
}

pub fn attach(pid: u32) {
    // Create message to send to service.
    let msg = service::AttachMessage::new(pid);
    let msg = bincode::serialize(&msg).expect("Fail to serialize message to send to service");

    // Send message to service.
    let mut pipe = OpenOptions::new()
        .write(true)
        .open("/var/run/escape-vpn/escape-vpn-service.pipe")
        .expect("Fail to open named pipe");

    println!("Pipe opened.");

    pipe.write_all(&msg)
        .expect("Fail to send message to service");
    pipe.flush().expect("Fail to flush pipe");
}
