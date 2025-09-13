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
                timestamp_fmt("[%Y-%m-%d %H:%M:%S.%3f]"),
                format!($($arg)*)
            ));
    };
}
/*
#[derive(Helper)]
#[helper(block)] */
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
                            color: Color::from_rgb_u8(0xff, 0, 0),
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

    let handler = std::thread::spawn({
        let window = main_window.as_weak();
        move || handle_event(window, receiver)
    });

    main_window.run()?;
    //log::debug!("Finished");
    s.exit();
    //s.send(MessageEvent::Exit);
    /* tokio::runtime::Builder::new_current_thread()
    .build()
    .unwrap()
    .block_on(thread)?; */
    handler.join().unwrap()?;
    matcher.join().unwrap()?;
    Ok(())
}
