use crossbeam_channel::{bounded, select, unbounded, Receiver, Select, Sender};
use show_image::{
    create_window, winit::platform::windows::HWND, Color, ImageInfo, ImageView, WindowOptions,
    WindowProxy,
};
use std::{
    collections::HashMap,
    thread::{self, JoinHandle},
};
use std::{mem, slice};
use windows_capture::{
    capture::{WindowsCaptureHandler, WindowsCaptureSettings},
    frame::{Frame, RGBA},
    window::Window,
};

pub struct StdinShell {
    rx_cmd: Receiver<StdinShellCommand>,
    tx_msg: Sender<StdinShellMessage>,
}

pub enum StdinShellCommand {}

pub enum StdinShellMessage {}

impl StdinShell {
    pub fn new() -> (Self, Sender<StdinShellCommand>, Receiver<StdinShellMessage>) {
        let (tx_cmd, rx_cmd) = unbounded();
        let (tx_msg, rx_msg) = unbounded();

        (Self { rx_cmd, tx_msg }, tx_cmd, rx_msg)
    }

    pub fn run(mut self) {
        // TODO
    }
}
