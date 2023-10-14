use std::collections::HashMap;

use crossbeam_channel::{Receiver, Sender};

use crate::{
    foreground_watcher::{ForegroundWatcherCommand, ForegroundWatcherMessage},
    image_viewer::{ImageViewerCommand, ImageViewerMessage},
    stdin_shell::{StdinShellCommand, StdinShellMessage},
    window_capture::{CapturedFrame, WindowCaptureMessage},
};

pub struct Driver {
    im_tx_cmd: Sender<ImageViewerCommand>,
    im_rx_msg: Receiver<ImageViewerMessage>,
    fw_tx_cmd: Sender<ForegroundWatcherCommand>,
    fw_rx_msg: Receiver<ForegroundWatcherMessage>,
    sh_tx_cmd: Sender<StdinShellCommand>,
    sh_rx_msg: Receiver<StdinShellMessage>,
    cap_rxs: HashMap<isize, (Receiver<WindowCaptureMessage>, Receiver<CapturedFrame>)>,
    current_hwnd: Option<HWND>,
}

impl Driver {
    pub fn new(
        im_tx_cmd: Sender<ImageViewerCommand>,
        im_rx_msg: Receiver<ImageViewerMessage>,
        fw_tx_cmd: Sender<ForegroundWatcherCommand>,
        fw_rx_msg: Receiver<ForegroundWatcherMessage>,
        sh_tx_cmd: Sender<StdinShellCommand>,
        sh_rx_msg: Receiver<StdinShellMessage>,
    ) -> Self {
        Self {
            im_tx_cmd,
            im_rx_msg,
            fw_tx_cmd,
            fw_rx_msg,
            sh_tx_cmd,
            sh_rx_msg,
            cap_rxs: HashMap::new(),
            current_hwnd: None,
        }
    }

    pub fn run(mut self) {
        let mut handlers: HashMap<HWND, Receiver<WindowCaptureMessage>> = HashMap::new();

        loop {
            if let Ok(msg) = self.im_rx_msg.try_recv() {
                self.handle_image_viewer_message(msg);
            }

            if let Ok(msg) = self.fw_rx_msg.try_recv() {
                self.handle_foreground_watcher_message(msg);
            }

            if let Ok(msg) = self.sh_rx_msg.try_recv() {
                self.handle_stdin_shell_message(msg);
            }

            let mut to_remove = vec![];
            for (_, (rx_msg, rx_frame)) in &mut self.cap_rxs {
                if let Ok(msg) = rx_msg.try_recv() {
                    match msg {
                        WindowCaptureMessage::Closed { hwnd } => {
                            to_remove.push(hwnd);
                            continue;
                        }
                    }
                }

                if let Ok(frame) = rx_frame.try_recv() {
                    if Some(frame.hwnd) == self.current_hwnd {
                        self.im_tx_cmd
                    }
                }
            }
        }
    }
}
