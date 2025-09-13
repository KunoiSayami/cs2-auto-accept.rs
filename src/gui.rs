use std::sync::OnceLock;

use std::sync::mpsc;

use slint::{Color, Model};

use crate::EXIT_SIGNAL;
use crate::real_main_guarder;
use crate::tools::timestamp_fmt;

slint::include_modules!();

pub(crate) static SENDER: OnceLock<MessageHelper> = OnceLock::new();
#[macro_export]
macro_rules! update_status {
    ($x:expr, $y: expr) => {
        gui::SENDER
            .get()
            .unwrap()
            .point($x, $y);
    };
    ($($arg:tt)*) => {
        gui::SENDER
            .get()
            .unwrap()
            .log(format!(
                "{} {}",
                timestamp_fmt("%Y-%m-%d %H:%M:%S.%3f"),
                format!($($arg)*)
            ));
    };
}
enum MessageEvent {
    Point(usize, usize),
    Log(String),
    Exit,
}

#[derive(Clone, Debug)]
pub struct MessageHelper {
    inner: mpsc::Sender<MessageEvent>,
}

impl MessageHelper {
    fn new() -> (Self, mpsc::Receiver<MessageEvent>) {
        let (s, r) = mpsc::channel();
        (Self { inner: s }, r)
    }

    pub(crate) fn log(&self, s: String) -> Option<()> {
        self.inner.send(MessageEvent::Log(s)).ok()
    }

    pub(crate) fn point(&self, x: usize, y: usize) -> Option<()> {
        self.inner.send(MessageEvent::Point(x, y)).ok()
    }

    fn exit(&self) -> Option<()> {
        self.inner.send(MessageEvent::Exit).ok()
    }
}

fn handle_event(
    window: slint::Weak<MainWindow>,
    receiver: mpsc::Receiver<MessageEvent>,
) -> anyhow::Result<()> {
    while let Ok(event) = receiver.recv() {
        match event {
            MessageEvent::Point(x, y) => {
                window
                    .upgrade_in_event_loop(move |w| {
                        let mut log_entries: Vec<_> = w.get_log_entries().iter().collect();
                        if log_entries.len() > 9 {
                            log_entries.remove(0);
                        }
                        log_entries.push(LogData {
                            color: Color::from_rgb_u8(68, 219, 46),
                            text: format!(
                                "{} x: {x}, y: {y}",
                                timestamp_fmt("[%Y-%m-%d %H:%M:%S.%3f]")
                            )
                            .into(),
                        });
                        let model = std::rc::Rc::new(slint::VecModel::from(log_entries));
                        w.set_log_entries(model.clone().into());
                    })
                    .unwrap();
            }
            MessageEvent::Log(s) => {
                window
                    .upgrade_in_event_loop(move |w| w.set_last_status(s.into()))
                    .unwrap();
            }
            MessageEvent::Exit => break,
        };

        //log::debug!("event: {s}");
    }
    EXIT_SIGNAL.set(true).ok();
    //log::debug!("exit");
    Ok(())
}

pub(crate) fn gui_entry(config: &String, force_distance: bool) -> anyhow::Result<()> {
    let (s, receiver) = MessageHelper::new();

    SENDER.set(s.clone()).unwrap();

    let matcher = std::thread::spawn({
        let config = config.to_string();
        move || real_main_guarder(&config, force_distance)
    });

    let main_window = MainWindow::new()?;

    main_window.set_save_image(crate::SAVE_IMAGE.load(std::sync::atomic::Ordering::Relaxed));
    main_window.set_dry_run(crate::DRY_RUN.load(std::sync::atomic::Ordering::Relaxed));

    main_window.on_dry_run_toggle(|state| {
        crate::DRY_RUN.store(state, std::sync::atomic::Ordering::Relaxed);
    });
    main_window.on_save_image_toggle(|state| {
        crate::SAVE_IMAGE.store(state, std::sync::atomic::Ordering::Relaxed);
    });

    let handler = std::thread::spawn({
        let window = main_window.as_weak();
        move || handle_event(window, receiver)
    });

    main_window.run()?;
    s.exit();
    handler.join().unwrap()?;
    matcher.join().unwrap()?;
    Ok(())
}
