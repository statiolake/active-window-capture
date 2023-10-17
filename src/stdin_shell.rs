use std::{ffi::OsString, fmt::Write as _, os::windows::prelude::OsStringExt, thread};

use crossbeam_channel::{unbounded, Receiver, Sender};
use rustyline::{DefaultEditor, ExternalPrinter};
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
    Output { message: String },
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

enum UserInput {
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
        let mut editor = DefaultEditor::new().expect("failed to prepare readline");
        let mut printer = editor
            .create_external_printer()
            .expect("failed to get printer");

        let (tx_input, rx_input) = unbounded();
        thread::spawn(move || keep_asking(editor, tx_input));

        loop {
            if let Ok(line) = rx_input.try_recv() {
                match self.parse_command(&line) {
                    Ok(UserInput::Nop) => {}
                    Ok(UserInput::Quit) => {
                        let _ = self.tx_msg.send(StdinShellMessage::QuitRequested);
                        break;
                    }
                    Ok(UserInput::AllowHWND(hwnds)) => {
                        let _ = self.tx_msg.send(StdinShellMessage::AllowHWND(hwnds));
                    }
                    Ok(UserInput::List) => {
                        let _ = self.tx_msg.send(StdinShellMessage::ListRequested);
                        printer
                            .print("Requesting allowed HWNDs, press Enter to refresh...".into())
                            .unwrap();
                    }
                    Ok(UserInput::Scan) => {
                        self.scan(&mut printer);
                    }
                    Err(e) => printer.print(format!("shell: {e}")).unwrap(),
                }
            };

            if let Ok(cmd) = self.rx_cmd.try_recv() {
                match cmd {
                    StdinShellCommand::Quit => break,
                    StdinShellCommand::Output { message } => {
                        printer.print(message).unwrap();
                    }
                }
            }
        }
    }

    fn scan<E: ExternalPrinter>(&mut self, printer: &mut E) {
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

        printer.print(buf).unwrap();

        self.scan_result = scan_result;
    }

    fn parse_command(&self, line: &str) -> Result<UserInput, String> {
        if line.trim().is_empty() {
            return Ok(UserInput::Nop);
        }

        let args: Vec<_> = line.split_whitespace().collect();

        if args[0] == "quit" {
            return Ok(UserInput::Quit);
        }

        if args[0] == "list" {
            return Ok(UserInput::List);
        }

        if args[0] == "scan" {
            return Ok(UserInput::Scan);
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

            return Ok(UserInput::AllowHWND(hwnds));
        }

        Err(format!("unknown command: {line}"))
    }
}

fn keep_asking(mut editor: DefaultEditor, sender: Sender<String>) {
    loop {
        let Ok(line) = editor.readline("shell> ") else {
            let _ = sender.send("quit".into());
            break;
        };

        let _ = sender.send(line);
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
