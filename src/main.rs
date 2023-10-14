use std::thread::{self};

use crate::{
    driver::Driver, foreground_watcher::ForegroundWatcher, image_viewer::ImageViewer,
    stdin_shell::StdinShell,
};

pub mod driver;
pub mod foreground_watcher;
pub mod image_viewer;
pub mod stdin_shell;
pub mod window_capture;

#[show_image::main]
fn main() {
    let (viewer, im_tx_cmd, im_rx_msg) = ImageViewer::new();
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
