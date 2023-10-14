use std::io::{self, Write};

use crossbeam_channel::{unbounded, Receiver, Sender};

pub struct StdinShell {
    rx_cmd: Receiver<StdinShellCommand>,
    tx_msg: Sender<StdinShellMessage>,
}

pub enum StdinShellCommand {
    Quit,
}

pub enum StdinShellMessage {
    QuitRequested,
}

enum Command {
    Nop,
    Quit,
}

impl StdinShell {
    pub fn new() -> (Self, Sender<StdinShellCommand>, Receiver<StdinShellMessage>) {
        let (tx_cmd, rx_cmd) = unbounded();
        let (tx_msg, rx_msg) = unbounded();

        (Self { rx_cmd, tx_msg }, tx_cmd, rx_msg)
    }

    pub fn run(self) {
        loop {
            if let Ok(cmd) = self.rx_cmd.try_recv() {
                match cmd {
                    StdinShellCommand::Quit => break,
                }
            }

            print!("shell> ");
            std::io::stdout().flush().unwrap();
            let stdin = io::stdin();
            let mut line = String::new();
            stdin.read_line(&mut line).unwrap();

            match parse_command(line.trim()) {
                Ok(Command::Nop) => {}
                Ok(Command::Quit) => {
                    let _ = self.tx_msg.send(StdinShellMessage::QuitRequested);
                    break;
                }
                Err(e) => {
                    eprintln!("shell: {e}");
                }
            }
        }
    }
}

fn parse_command(line: &str) -> Result<Command, String> {
    if line.is_empty() {
        return Ok(Command::Nop);
    }

    if line == "quit" {
        return Ok(Command::Quit);
    }

    Err(format!("unknown command: {line}"))
}
