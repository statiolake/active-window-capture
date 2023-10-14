use std::io::{self, Write};

use crossbeam_channel::{unbounded, Receiver, Sender};
use windows::Win32::Foundation::HWND;

pub struct StdinShell {
    rx_cmd: Receiver<StdinShellCommand>,
    tx_msg: Sender<StdinShellMessage>,
}

pub enum StdinShellCommand {
    Quit,
}

pub enum StdinShellMessage {
    QuitRequested,
    AllowHWND(Vec<HWND>),
}

enum Command {
    Nop,
    Quit,
    AllowHWND(Vec<HWND>),
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
                Ok(Command::AllowHWND(hwnds)) => {
                    let _ = self.tx_msg.send(StdinShellMessage::AllowHWND(hwnds));
                }
                Err(e) => {
                    eprintln!("shell: {e}");
                }
            }
        }
    }
}

fn parse_command(line: &str) -> Result<Command, String> {
    if line.trim().is_empty() {
        return Ok(Command::Nop);
    }

    let args: Vec<_> = line.split_whitespace().collect();
    if args[0] == "quit" {
        return Ok(Command::Quit);
    }

    if args[0].starts_with("allow") {
        if args.len() == 1 {
            return Err(format!("allow needs at least one HWND"));
        }

        let Ok(hwnds) = args[1..].iter().map(|arg| arg.parse().map(HWND)).collect() else {
            return Err(format!("unknown HWND in allow"));
        };

        return Ok(Command::AllowHWND(hwnds));
    }

    Err(format!("unknown command: {line}"))
}
