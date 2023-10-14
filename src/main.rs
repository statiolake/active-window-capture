use crossbeam_channel::{bounded, select, unbounded, Receiver, Select, Sender};
use foreground_watcher::{ForegroundWatcherCommand, ForegroundWatcherMessage};
use image_viewer::{ImageViewerCommand, ImageViewerMessage};
use show_image::{
    create_window, winit::platform::windows::HWND, Color, ImageInfo, ImageView, WindowOptions,
    WindowProxy,
};
use std::{
    collections::HashMap,
    thread::{self, JoinHandle},
};
use std::{mem, slice};
use stdin_shell::{StdinShellCommand, StdinShellMessage};
use window_capture::{WindowCapture, WindowCaptureMessage};
use windows_capture::{
    capture::{WindowsCaptureHandler, WindowsCaptureSettings},
    frame::{Frame, RGBA},
    window::Window,
};

use crate::{
    foreground_watcher::ForegroundWatcher, image_viewer::ImageViewer, stdin_shell::StdinShell,
};

pub mod driver;
pub mod foreground_watcher;
pub mod image_viewer;
pub mod stdin_shell;
pub mod window_capture;

#[show_image::main]
fn main() {
    let (tx_frame, rx_frame) = bounded(5);

    let (viewer, im_tx_cmd, im_rx_msg) = ImageViewer::new(rx_frame);
    let viewer = thread::spawn(move || viewer.run());

    let (watcher, fw_tx_cmd, fw_rx_msg) = ForegroundWatcher::new();
    let watcher = thread::spawn(move || watcher.run());

    let (shell, sh_tx_cmd, sh_rx_msg) = StdinShell::new();
    let shell = thread::spawn(move || shell.run());

    let driver = Driver::new(
        im_tx_cmd, im_rx_msg, fw_tx_cmd, fw_rx_msg, sh_tx_cmd, sh_rx_msg,
    );

    driver.run();

    viewer.join().unwrap();
    watcher.join().unwrap();
    shell.join().unwrap();
}
