use std::{thread, time::Duration};

use crossbeam_channel::{unbounded, Receiver, Sender};
use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::GetForegroundWindow};

pub struct ForegroundWatcher {
    rx_cmd: Receiver<ForegroundWatcherCommand>,
    tx_msg: Sender<ForegroundWatcherMessage>,
    old_hwnd: Option<HWND>,
}

pub enum ForegroundWatcherCommand {
    Quit,
}

pub enum ForegroundWatcherMessage {
    WindowChanged { hwnd: HWND },
}

impl ForegroundWatcher {
    pub fn new() -> (
        Self,
        Sender<ForegroundWatcherCommand>,
        Receiver<ForegroundWatcherMessage>,
    ) {
        let (tx_cmd, rx_cmd) = unbounded();
        let (tx_msg, rx_msg) = unbounded();

        (
            Self {
                rx_cmd,
                tx_msg,
                old_hwnd: None,
            },
            tx_cmd,
            rx_msg,
        )
    }

    pub fn run(mut self) {
        loop {
            if let Ok(msg) = self.rx_cmd.try_recv() {
                match msg {
                    ForegroundWatcherCommand::Quit => break,
                }
            }

            let hwnd = unsafe { GetForegroundWindow() };
            if Some(hwnd) != self.old_hwnd {
                self.old_hwnd = Some(hwnd);
                let _ = self
                    .tx_msg
                    .send(ForegroundWatcherMessage::WindowChanged { hwnd });
            }

            thread::sleep(Duration::from_millis(100));
        }
    }
}
