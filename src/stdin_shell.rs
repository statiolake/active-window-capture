use std::io::{self, Write};

use crossbeam_channel::{unbounded, Receiver, Sender};
use windows::Win32::Foundation::HWND;

pub struct StdinShell {
    rx_cmd: Receiver<StdinShellCommand>,
    tx_msg: Sender<StdinShellMessage>,
}

pub enum StdinShellCommand {
    Quit,
    ListResponse { allowed_hwnds: Vec<HWND> },
}

pub enum StdinShellMessage {
    QuitRequested,
    AllowHWND(Vec<HWND>),
    ListRequested,
}

enum Command {
    Nop,
    Quit,
    AllowHWND(Vec<HWND>),
    List,
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
                    StdinShellCommand::ListResponse { allowed_hwnds } => {
                        let mut s = String::new();
                        s.push_str("Got allowed HWNDs:\n");
                        for hwnd in allowed_hwnds {
                            s.push_str(&format!("| {}", hwnd.0));
                        }
                        // エラー出力などに紛れないよう一括出力
                        println!("{}", s);
                    }
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
                Ok(Command::List) => {
                    let _ = self.tx_msg.send(StdinShellMessage::ListRequested);
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

    if args[0] == "list" {
        println!("Requesting allowed HWNDs, press Enter to refresh...");
        return Ok(Command::List);
    }

    if args[0].starts_with("allow") {
        if args.len() == 1 {
            return Err("allow needs at least one HWND".into());
        }

        let Ok(hwnds) = args[1..].iter().map(|arg| arg.parse().map(HWND)).collect() else {
            return Err("unknown HWND in allow".into());
        };

        return Ok(Command::AllowHWND(hwnds));
    }

    Err(format!("unknown command: {line}"))
}
