use crossbeam_channel::{unbounded, Receiver, Sender};
use show_image::winit::platform::windows::HWND;

pub struct ForegroundWatcher {
    rx_cmd: Receiver<ForegroundWatcherCommand>,
    tx_msg: Sender<ForegroundWatcherMessage>,
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

        (Self { rx_cmd, tx_msg }, tx_cmd, rx_msg)
    }

    pub fn run(mut self) {
        // TODO
    }
}
