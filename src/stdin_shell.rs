use std::{
    ffi::OsString,
    fmt::Write as _,
    io::{self, Write as _},
    os::windows::prelude::OsStringExt,
};

use crossbeam_channel::{unbounded, Receiver, Sender};
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM},
    UI::WindowsAndMessaging::{EnumWindows, GetWindowLongW, GetWindowTextW, GWL_STYLE},
};

pub struct StdinShell {
    rx_cmd: Receiver<StdinShellCommand>,
    tx_msg: Sender<StdinShellMessage>,
    scan_result: Vec<ScanEntry>,
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

struct ScanEntry {
    alias: Option<char>,
    hwnd: HWND,
    title: String,
}

enum Command {
    Nop,
    Quit,
    AllowHWND(Vec<HWND>),
    List,
    Scan,
}

impl StdinShell {
    pub fn new() -> (Self, Sender<StdinShellCommand>, Receiver<StdinShellMessage>) {
        let (tx_cmd, rx_cmd) = unbounded();
        let (tx_msg, rx_msg) = unbounded();

        (
            Self {
                rx_cmd,
                tx_msg,
                scan_result: vec![],
            },
            tx_cmd,
            rx_msg,
        )
    }

    pub fn run(mut self) {
        loop {
            if let Ok(cmd) = self.rx_cmd.try_recv() {
                match cmd {
                    StdinShellCommand::Quit => break,
                    StdinShellCommand::ListResponse { allowed_hwnds } => {
                        let mut buf = String::new();
                        writeln!(buf, "Got allowed HWNDs:").unwrap();
                        for hwnd in allowed_hwnds {
                            writeln!(buf, "| {}", hwnd.0).unwrap();
                        }

                        // エラー出力などに紛れないよう一括出力
                        println!("{buf}");
                    }
                }
            }

            print!("shell> ");
            std::io::stdout().flush().unwrap();
            let stdin = io::stdin();
            let mut line = String::new();
            stdin.read_line(&mut line).unwrap();

            match self.parse_command(line.trim()) {
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
                    println!("Requesting allowed HWNDs, press Enter to refresh...");
                }
                Ok(Command::Scan) => {
                    self.scan();
                }
                Err(e) => {
                    eprintln!("shell: {e}");
                }
            }
        }
    }

    fn scan(&mut self) {
        let mut scan_result = enumerate_windows();
        for (alias, entry) in ('A'..='Z').zip(&mut scan_result) {
            entry.alias = Some(alias);
        }

        // 途中で表示が割り込まれてほしくないのでまとめて
        let mut buf = String::new();
        writeln!(&mut buf, "Opened windows:").unwrap();
        for entry in &scan_result {
            writeln!(
                &mut buf,
                "| {}) [{:>8}] {}",
                entry.alias.unwrap_or(' '),
                entry.hwnd.0,
                entry.title
            )
            .unwrap();
        }
        println!("{buf}");

        self.scan_result = scan_result;
    }

    fn parse_command(&self, line: &str) -> Result<Command, String> {
        if line.trim().is_empty() {
            return Ok(Command::Nop);
        }

        let args: Vec<_> = line.split_whitespace().collect();

        if args[0] == "quit" {
            return Ok(Command::Quit);
        }

        if args[0] == "list" {
            return Ok(Command::List);
        }

        if args[0] == "scan" {
            return Ok(Command::Scan);
        }

        if args[0].starts_with("allow") {
            if args.len() == 1 {
                return Err("allow needs at least one HWND".into());
            }

            let mut hwnds = vec![];
            'next_arg: for arg in &args[1..] {
                if arg.len() == 1 {
                    for entry in &self.scan_result {
                        if entry.alias == Some(arg.chars().next().unwrap()) {
                            hwnds.push(entry.hwnd);
                            continue 'next_arg;
                        }
                    }
                }

                let Ok(hwnd) = arg.parse() else {
                    return Err(format!("unknown HWND {arg} in allow"));
                };

                hwnds.push(HWND(hwnd));
            }

            return Ok(Command::AllowHWND(hwnds));
        }

        Err(format!("unknown command: {line}"))
    }
}

fn enumerate_windows() -> Vec<ScanEntry> {
    let mut windows: Vec<ScanEntry> = vec![];
    unsafe {
        EnumWindows(
            Some(enumerate_callback),
            LPARAM(&mut windows as *mut _ as isize),
        )
        .unwrap()
    };

    // 普通のウィンドウに限る
    windows
        .into_iter()
        .filter(|win| {
            let style = unsafe { GetWindowLongW(win.hwnd, GWL_STYLE) }; // GWL_STYLE

            // WS_VISIBLEとWS_CAPTION
            (style & 0x10C00000) == 0x10C00000
        })
        .collect()
}

unsafe extern "system" fn enumerate_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = unsafe { &mut *(lparam.0 as *mut Vec<ScanEntry>) };

    let mut title_u16 = vec![0; 1024];
    GetWindowTextW(hwnd, &mut title_u16);
    let title = OsString::from_wide(&title_u16)
        .to_string_lossy()
        .into_owned();
    windows.push(ScanEntry {
        alias: None,
        hwnd,
        title,
    });

    BOOL(1)
}
