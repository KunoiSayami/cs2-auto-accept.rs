mod configure;
mod definitions;
mod impls;
mod matcher;
mod target_5e;
mod target_main;
mod tools;
mod types;

use std::{
    io::Write,
    sync::{OnceLock, atomic::AtomicBool, mpsc},
    thread::sleep,
    time::{Duration, Instant},
};

use clap::{Command, arg};
use configure::{Configure, Point};
use image::{DynamicImage, ImageBuffer, Rgb};
use rayon::iter::ParallelIterator;
use sysinfo::{ProcessRefreshKind, RefreshKind};
use tools::{load_and_display, test_image, timestamp_fmt};
use xcap::Monitor;

use crate::{
    impls::{get_pos, move_mouse_click},
    matcher::Matcher,
    types::MatchOptions,
};

#[cfg(feature = "distance")]
mod distance;
#[cfg(not(feature = "distance"))]
mod distance {
    pub(crate) fn calc_color_distance(_: &String, _: bool) -> ! {
        unimplemented!("To use this function, enable \"distance\" feature")
    }
}

static TEST_MODE: AtomicBool = AtomicBool::new(false);
static SAVE_IMAGE: AtomicBool = AtomicBool::new(false);
static EXIT_SIGNAL: OnceLock<bool> = OnceLock::new();

const X_LIMIT: usize = 10;
const Y_LIMIT: usize = 8;

macro_rules! print_inline {
    ($($arg:tt)*) => {{
        print!("\r{}", timestamp_fmt("[%Y-%m-%d %H:%M:%S] "));
        print!($($arg)*);
        print!("\r");
        {
            std::io::stdout().lock().flush().unwrap();
        }
    }};
}

macro_rules! sleep_until_exit {
    ($time:expr) => {
        if sleep_until_exit($time) {
            break;
        }
    };
}

#[derive(Debug)]
enum SearchResult {
    Found(usize, usize),
    NotFound,
}

enum CheckResult {
    NeedProcess,
    // Wait for another check
    NoNeedProcess,
    // Call next check (if there is)
    Next,
}

type BasicImageType = Rgb<u8>;
type SubImageType = Vec<u8>;
type ImageType = ImageBuffer<BasicImageType, SubImageType>;

fn determine_point(monitor: Monitor, is_5e: bool) -> anyhow::Result<Point> {
    let x = monitor.x()?;
    let y = monitor.y()? - if is_5e { -50 } else { 100 };
    let height = monitor.height()? as i32;
    let width = monitor.width()? as i32;
    let h = 100;
    let w = 200;

    let mid_x = x + width / 2;
    let mid_y = y + height / 2;

    Ok(Point::new(mid_x - w, mid_y - h, mid_x + w, mid_y + h))
}

fn screen_cap(point: Option<Point>, is_5e: bool) -> anyhow::Result<(Point, ImageType)> {
    let start = Instant::now();
    let monitors = Monitor::all().unwrap();

    for monitor in monitors {
        if !monitor.is_primary().unwrap_or_default() {
            continue;
        }
        let real_point = match point {
            Some(point) => point,
            None => determine_point(monitor.clone(), is_5e)?,
        };

        let image = monitor.capture_region(
            real_point.x() as u32,
            real_point.y() as u32,
            real_point.width() as u32,
            real_point.height() as u32,
        )?;
        //log::debug!("{real_point:?}");

        //return Ok(DynamicImage::from(image).into_rgb8());
        log::trace!("elapsed: {:?}", start.elapsed());
        if SAVE_IMAGE.load(std::sync::atomic::Ordering::Relaxed) {
            image.save(format!("{}.png", timestamp_fmt("%Y-%m-%d_%H-%M-%S")))?;
        }
        return Ok((real_point, DynamicImage::from(image).into_rgb8()));
    }
    Err(anyhow::anyhow!("Not found primary monitor"))
}

fn process_area(
    area: &ImageType,
    template: &Matcher,
    options: MatchOptions,
) -> (Vec<Vec<bool>>, usize) {
    let (pic_x, pic_y) = area.dimensions();
    let mut buff = vec![vec![false; pic_y as usize]; pic_x as usize];
    let iter = area.par_enumerate_pixels();
    let mut count = 0;
    let (sender, recv) = mpsc::channel();

    //let beg = Instant::now();
    iter.for_each_init(
        || sender.clone(),
        |s, (x, y, p)| {
            if template.check(p, options.force_distance()) {
                s.send((x, y)).ok();
            }
        },
    );
    drop(sender);

    while let Ok((x, y)) = recv.recv() {
        buff[x as usize][y as usize] = true;
        count += 1;
    }
    //log::debug!("Elapsed: {:?}", beg.elapsed());
    (buff, count)
}

fn match_algorithm(
    point: Point,
    buff: &[Vec<bool>],
    (pic_x, pic_y): (u32, u32),
    options: MatchOptions,
) -> SearchResult {
    let x_start = options.limit_x() / 2;
    let x_end = pic_x as usize - x_start;
    let y_start = options.limit_y() / 2;
    let y_end = pic_y as usize - y_start;

    for x in x_start..x_end {
        for y in y_start..y_end {
            let original_x = x - x_start;
            let original_y = y - y_start;
            let mut r = true;
            //let mut matched = 0;
            'outer: for buff in buff.iter().skip(original_x).take(options.limit_x()) {
                for element in buff.iter().skip(original_y).take(options.limit_y()) {
                    if !*element {
                        /* if matched > 0 {
                            log::debug!("Matched: {matched}");
                        } */
                        r = false;
                        break 'outer;
                    }
                    //matched += 1;
                }
            }
            if r {
                return SearchResult::Found(x + point.x() as usize, y + point.y() as usize);
            }
        }
    }
    SearchResult::NotFound
}

pub(crate) fn check_image_match(
    point: Option<Point>,
    is_5e: bool,
    template: &Matcher,
    options: MatchOptions,
) -> anyhow::Result<SearchResult> {
    print_inline!("Capture screen             ");
    let (point, current_screen) = screen_cap(point, is_5e)?;
    print_inline!("Marking area into Vec<bool>");
    let (buff, count) = process_area(&current_screen, template, options);
    if count < options.limit_x() * options.limit_y() {
        return Ok(SearchResult::NotFound);
    }
    print_inline!("Checking point of interest");
    //let instant = Instant::now();
    let ret = match_algorithm(point, &buff, current_screen.dimensions(), options);
    //log::debug!("elapsed: {:?}", instant.elapsed());
    Ok(ret)
}

fn display_mouse() -> anyhow::Result<()> {
    get_pos()
}

fn handle_target(result: SearchResult) -> anyhow::Result<bool> {
    if let SearchResult::Found(pos1, pos2) = result {
        log::debug!("Mouse point: x: {pos1}, y: {pos2}");
        move_mouse_click(
            pos1 as i32,
            pos2 as i32,
            TEST_MODE.load(std::sync::atomic::Ordering::Relaxed),
        )?;

        return Ok(true);
    }

    Ok(false)
}

fn sleep_until_exit(second: usize) -> bool {
    for _ in 0..(second * 2) {
        if EXIT_SIGNAL.get().is_some() {
            return true;
        }
        sleep(Duration::from_millis(500));
    }
    false
}

fn real_main(config: &String, force_distance: bool) -> anyhow::Result<()> {
    let config = Configure::load(config).unwrap_or_default();
    let mut sys = sysinfo::System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );

    let options = MatchOptions::new(force_distance, X_LIMIT, Y_LIMIT);

    log::info!("Starting listening");

    loop {
        sys.refresh_all();

        match target_5e::check_need_handle(sys.processes()) {
            CheckResult::NeedProcess => {
                print_inline!("Match 5e     ");

                let ret =
                    check_image_match(config.e5(), true, &target_5e::MATCH_TEMPLATE, options)?;
                handle_target(ret)?;
            }
            CheckResult::NoNeedProcess => {
                print_inline!("User is playing     ");
                sleep_until_exit!(60);
                continue;
            }
            CheckResult::Next => {}
        }

        match target_main::check_primary_exec(sys.processes()) {
            CheckResult::NeedProcess => {
                print_inline!("Match CS2     ");
                let ret =
                    check_image_match(config.cs2(), false, &target_main::MATCH_TEMPLATE, options)?;
                handle_target(ret)?;
            }
            CheckResult::NoNeedProcess => {
                unimplemented!()
            }
            CheckResult::Next => {}
        }
        //log::debug!("Next tick");
        print_inline!("Sleep                      ");
        if !TEST_MODE.load(std::sync::atomic::Ordering::Relaxed) {
            sleep_until_exit!(3);
        } else {
            sleep_until_exit!(2);
        }
    }
    log::info!("User exit");
    Ok(())
}

fn real_main_guarder(config: &String, force_distance: bool) -> anyhow::Result<()> {
    let mut err = None;
    while EXIT_SIGNAL.get().is_none() {
        if let Err(e) = real_main(config, force_distance)
            .inspect_err(|e| log::error!("Main thread error: {e:?}"))
        {
            err.replace(e);
        }
    }
    if let Some(e) = err { Err(e) } else { Ok(()) }
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("enigo", log::LevelFilter::Warn)
        .init();
    ctrlc::set_handler(|| {
        EXIT_SIGNAL.set(true).unwrap();
    })
    .unwrap();
    let matches = clap::command!()
        .args(&[
            arg!([CONFIG] "Configure file").default_value("config.toml"),
            arg!(--"dry-run" "Dry run (do not click)"),
            arg!(--"save-image" "Save image each time take"),
            arg!(--"force-distance" "Use distance algorithm to check image"),
        ])
        .subcommand(Command::new("mouse"))
        .subcommand(Command::new("get-color").args(&[
            arg!(<FILE> ... "Image file"),
            arg!(--output <OUTPUT> "Output file").default_missing_value("output.rs"),
        ]))
        .subcommand(
            Command::new("distance")
                .args(&[
                    arg!(<FILE> "RGB file, generate by get-color command"),
                    arg!(--"read-only" "No write, just read"),
                ])
                .hide(cfg!(feature = "distance")),
        )
        .subcommand(
            Command::new("test").args(&[arg!(<FILE> "Test image"), arg!(--"5e" "Enable 5e match")]),
        )
        .get_matches();

    if matches.get_flag("dry-run") {
        TEST_MODE.store(true, std::sync::atomic::Ordering::Relaxed);
        log::debug!("Dry running");
    }

    SAVE_IMAGE.store(
        matches.get_flag("save-image"),
        std::sync::atomic::Ordering::Relaxed,
    );

    match matches.subcommand() {
        Some(("mouse", _)) => display_mouse(),
        Some(("get-color", matches)) => load_and_display(
            &matches.get_many::<String>("FILE").unwrap(),
            matches.get_one("output"),
        ),
        Some(("distance", matches)) => distance::calc_color_distance(
            matches.get_one::<String>("FILE").unwrap(),
            matches.get_flag("read-only"),
        ),
        Some(("test", matches)) => test_image(
            matches.get_one("FILE").unwrap(),
            matches.get_flag("5e"),
            matches.get_flag("force-distance"),
        ),
        _ => real_main_guarder(
            matches.get_one("CONFIG").unwrap(),
            matches.get_flag("force-distance"),
        ),
    }
}
