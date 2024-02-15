use color_eyre::eyre::{eyre, Result};
use std::{
    cell::Cell,
    collections::HashMap,
    sync::{mpsc::Sender, Arc, Mutex, OnceLock},
    thread::JoinHandle,
};

type Processes = Arc<Mutex<HashMap<u32, (Cell<Option<JoinHandle<()>>>, Sender<()>)>>>;

static PROCESSES: OnceLock<Processes> = OnceLock::new();

pub fn add_process(pid: u32, join_handler: JoinHandle<()>, exit_sender: Sender<()>) -> Result<()> {
    let processes = PROCESSES.get_or_init(Default::default);
    let Ok(mut processes) = processes.lock() else {
        return Err(eyre!("Fail to lock processes collection."));
    };

    processes.insert(pid, (Cell::new(Some(join_handler)), exit_sender));

    Ok(())
}

pub fn remove_process_and_trigger_exit(pid: u32) -> Result<bool> {
    let processes = PROCESSES.get_or_init(Default::default);

    let Ok(processes) = processes.lock() else {
        return Err(eyre!("Fail to lock processes collection."));
    };

    let Some((join_handler, exit_sender)) = processes.get(&pid) else {
        return Ok(false);
    };

    // Send signal to stop monitoring process.
    exit_sender.send(())?;

    // Wait for process monitoring thread to finish.
    let Some(join_handler) = join_handler.take() else {
        return Err(eyre!("Fail to take thread handler."));
    };
    join_handler.join().unwrap();

    Ok(true)
}
