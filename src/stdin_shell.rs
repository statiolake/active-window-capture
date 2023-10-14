use std::{thread, time::Duration};

use crossbeam_channel::{unbounded, Receiver, Sender};

pub struct StdinShell {
    rx_cmd: Receiver<StdinShellCommand>,
    _tx_msg: Sender<StdinShellMessage>,
}

pub enum StdinShellCommand {
    Quit,
}

pub enum StdinShellMessage {}

impl StdinShell {
    pub fn new() -> (Self, Sender<StdinShellCommand>, Receiver<StdinShellMessage>) {
        let (tx_cmd, rx_cmd) = unbounded();
        let (tx_msg, rx_msg) = unbounded();

        (
            Self {
                rx_cmd,
                _tx_msg: tx_msg,
            },
            tx_cmd,
            rx_msg,
        )
    }

    pub fn run(self) {
        loop {
            if let Ok(cmd) = self.rx_cmd.try_recv() {
                match cmd {
                    StdinShellCommand::Quit => break,
                }
            }
            // TODO
            thread::sleep(Duration::from_millis(100));
        }
    }
}
