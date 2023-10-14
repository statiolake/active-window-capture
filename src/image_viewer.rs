use crossbeam_channel::{unbounded, Receiver, Sender};
use show_image::{create_window, Color, ImageInfo, ImageView, WindowOptions, WindowProxy};

use crate::window_capture::CapturedFrame;

pub struct ImageViewer {
    rx_cmd: Receiver<ImageViewerCommand>,
    tx_msg: Sender<ImageViewerMessage>,
    is_running: bool,
}

pub enum ImageViewerCommand {
    Update(CapturedFrame),
    Quit,
}

pub enum ImageViewerMessage {
    Closed,
}

impl ImageViewer {
    pub fn new() -> (
        ImageViewer,
        Sender<ImageViewerCommand>,
        Receiver<ImageViewerMessage>,
    ) {
        let (tx_cmd, rx_cmd) = unbounded();
        let (tx_msg, rx_msg) = unbounded();

        (
            ImageViewer {
                rx_cmd,
                tx_msg,
                is_running: false,
            },
            tx_cmd,
            rx_msg,
        )
    }

    pub fn run(mut self) {
        let Ok(window) = create_window(
            "Capture",
            WindowOptions::new()
                .set_background_color(Color::rgb(0.0, 1.0, 0.0))
                .set_fullscreen(true)
                .set_preserve_aspect_ratio(true)
                .set_default_controls(false),
        ) else {
            return;
        };

        self.is_running = true;

        while self.is_running {
            if let Ok(cmd) = self.rx_cmd.recv() {
                self.handle_command(&window, cmd);
            }
        }
    }

    fn handle_command(&mut self, window: &WindowProxy, command: ImageViewerCommand) {
        match command {
            ImageViewerCommand::Update(frame) => {
                let image =
                    ImageView::new(ImageInfo::rgba8(frame.width, frame.height), &frame.bytes);
                if window.set_image("capture", image).is_err() {
                    let _ = self.tx_msg.send(ImageViewerMessage::Closed);
                }
            }
            ImageViewerCommand::Quit => self.is_running = false,
        }
    }
}
