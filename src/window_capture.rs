use crossbeam_channel::{unbounded, Receiver, Sender};
use std::{
    mem, slice,
    time::{Duration, Instant},
};
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
        let tx_msg = self.tx_msg.clone();

        let settings = WindowsCaptureSettings::new(
            Window::from_hwnd(self.hwnd),
            true,
            false,
            WindowCaptureArgs {
                tx_msg: self.tx_msg,
                hwnd: self.hwnd,
                tx_frame: self.tx_frame,
                fps: 30,
            },
        );

        if let Err(e) = Handler::start(settings) {
            eprintln!("failed to capture {:x}: {e:?}", self.hwnd.0);
            let _ = tx_msg.send(WindowCaptureMessage::Closed { hwnd: self.hwnd });
        }
    }
}

pub struct WindowCaptureArgs {
    tx_msg: Sender<WindowCaptureMessage>,
    hwnd: HWND,
    tx_frame: Sender<CapturedFrame>,
    fps: u64,
}

pub struct Handler {
    args: WindowCaptureArgs,
    next_update: Instant,
}

impl Handler {
    fn compute_next_update(&mut self) {
        let mut next_update = self.next_update + Duration::from_millis(1000 / self.args.fps);
        let now = Instant::now();
        if now < next_update {
            next_update = now + Duration::from_millis(1000 / self.args.fps);
        }
        self.next_update = next_update;
    }
}

impl WindowsCaptureHandler for Handler {
    type Flags = WindowCaptureArgs;

    fn new(args: Self::Flags) -> Self {
        Self {
            args,
            next_update: Instant::now(),
        }
    }

    fn on_frame_arrived(&mut self, frame: &Frame) {
        if Instant::now() < self.next_update {
            return;
        }

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

        let _ = self.args.tx_frame.send(CapturedFrame {
            hwnd: self.args.hwnd,
            width: buffer.width(),
            height: buffer.height(),
            bytes,
        });

        self.compute_next_update();
    }

    fn on_closed(&mut self) {
        let _ = self.args.tx_msg.send(WindowCaptureMessage::Closed {
            hwnd: self.args.hwnd,
        });
    }
}
