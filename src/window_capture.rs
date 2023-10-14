use crossbeam_channel::{unbounded, Receiver, Sender};
use std::{mem, slice};
use windows::Win32::Foundation::HWND;
use windows_capture::{
    capture::{WindowsCaptureHandler, WindowsCaptureSettings},
    frame::{Frame, RGBA},
    window::Window,
};

pub struct CapturedFrame {
    pub hwnd: HWND,
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
}

pub struct WindowCapture {
    _rx_cmd: Receiver<WindowCaptureCommand>,
    tx_msg: Sender<WindowCaptureMessage>,
    hwnd: HWND,
    tx_frame: Sender<CapturedFrame>,
}

pub enum WindowCaptureCommand {}

pub enum WindowCaptureMessage {
    Closed { hwnd: HWND },
}

impl WindowCapture {
    pub fn new(
        hwnd: HWND,
        tx_frame: Sender<CapturedFrame>,
    ) -> (
        WindowCapture,
        Sender<WindowCaptureCommand>,
        Receiver<WindowCaptureMessage>,
    ) {
        let (tx_cmd, rx_cmd) = unbounded();
        let (tx_msg, rx_msg) = unbounded();

        (
            WindowCapture {
                _rx_cmd: rx_cmd,
                tx_msg,
                hwnd,
                tx_frame,
            },
            tx_cmd,
            rx_msg,
        )
    }

    pub fn run(self) {
        let settings = WindowsCaptureSettings::new(
            Window::from_hwnd(self.hwnd),
            true,
            false,
            (self.tx_msg, self.hwnd, self.tx_frame),
        );
        Handler::start(settings).unwrap();
    }
}

pub struct Handler {
    rx_msg: Sender<WindowCaptureMessage>,
    tx_frame: Sender<CapturedFrame>,
    hwnd: HWND,
}

impl WindowsCaptureHandler for Handler {
    type Flags = (Sender<WindowCaptureMessage>, HWND, Sender<CapturedFrame>);

    fn new((rx_msg, hwnd, tx_frame): Self::Flags) -> Self {
        Self {
            rx_msg,
            hwnd,
            tx_frame,
        }
    }

    fn on_frame_arrived(&mut self, frame: &Frame) {
        let buffer = frame.buffer().unwrap();

        // バッファは画像幅を32の倍数に切り上げて送ってくるらしいので、バッファの幅を計算する。
        let width32 = ((buffer.width() + 31) / 32 * 32) as usize;

        let pixels = buffer.pixels();

        // 画像のうち「32の倍数に満たなかったあまり部分」には適当なごみデータが入っているようなの
        // で、ごみデータ部分を削除する。
        let mut bytes = Vec::with_capacity(
            buffer.width() as usize * buffer.height() as usize * mem::size_of::<RGBA>(),
        );

        for i in 0..buffer.height() as usize {
            let start = i * width32;
            let row_pixels = &pixels[start..(start + buffer.width() as usize)];
            let row_bytes = unsafe {
                slice::from_raw_parts(
                    row_pixels as *const _ as *const u8,
                    mem::size_of_val(row_pixels),
                )
            };
            bytes.extend(row_bytes);
        }

        let _ = self.tx_frame.send(CapturedFrame {
            hwnd: self.hwnd,
            width: buffer.width(),
            height: buffer.height(),
            bytes,
        });
    }

    fn on_closed(&mut self) {
        let _ = self
            .rx_msg
            .send(WindowCaptureMessage::Closed { hwnd: self.hwnd });
    }
}
