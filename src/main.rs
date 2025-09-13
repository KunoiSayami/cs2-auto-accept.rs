#![cfg_attr(feature = "gui", windows_subsystem = "windows")]

mod configure;
mod definitions;
#[cfg(feature = "gui")]
mod gui;
mod matcher;
mod not_impl;
mod platform_impl;
mod target_5e;
mod target_main;
mod tools;
mod types;

use std::{
    sync::{OnceLock, atomic::AtomicBool, mpsc},
    thread::sleep,
    time::{Duration, Instant},
};

use clap::{Command, arg, builder::PossibleValue};
use configure::Configure;
use image::{DynamicImage, ImageBuffer, Rgb};
use rayon::iter::ParallelIterator;
use sysinfo::{ProcessRefreshKind, RefreshKind};
use tools::{continue_test_area, load_and_display, test_image, timestamp_fmt};
use xcap::Monitor;

use crate::{
    matcher::Matcher,
    platform_impl::{get_pos, move_mouse_click},
    types::{MatchOptions, Point, PointOption},
};

#[cfg(feature = "distance")]
mod distance;

#[cfg(feature = "jpeg")]
use crate::matcher::dir_match;

#[allow(unused)]
use crate::not_impl::*;

static TEST_MODE: AtomicBool = AtomicBool::new(false);
static SAVE_IMAGE: AtomicBool = AtomicBool::new(false);
static EXIT_SIGNAL: OnceLock<bool> = OnceLock::new();

const X_LIMIT: usize = 10;
const Y_LIMIT: usize = 8;
const X_LIMIT_5E: usize = 26;
const Y_LIMIT_5E: usize = 12;

#[cfg(not(feature = "gui"))]
macro_rules! print_inline {
    ($($arg:tt)*) => {{
        print!("\r{}", timestamp_fmt("[%Y-%m-%d %H:%M:%S.%3f] "));
        print!($($arg)*);
        print!("\r");
        {
            std::io::Write::flush(&mut std::io::stdout().lock()).unwrap();
        }
    }};
}

#[cfg(feature = "gui")]
macro_rules! print_inline {
    ($($arg:tt)*) => {{
        update_status!($($arg)*);
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

#[must_use]
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

fn screen_cap(point: PointOption, is_5e: bool) -> anyhow::Result<(Point, ImageType)> {
    let start = Instant::now();
    let monitors = Monitor::all().unwrap();

    for monitor in monitors {
        if !monitor.is_primary().unwrap_or_default() {
            continue;
        }
        let real_point = match point {
            PointOption::Some(point) => point,
            PointOption::Transform(func) => func(monitor.clone()),
            PointOption::None => determine_point(monitor.clone(), is_5e)?,
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
            image.save(format!("{}.png", timestamp_fmt("%Y-%m-%d_%H-%M-%S-%3f")))?;
        }
        return Ok((real_point, DynamicImage::from(image).into_rgb8()));
    }
    Err(anyhow::anyhow!("Not found primary monitor"))
}

#[must_use]
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

#[must_use]
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
    point: PointOption,
    is_5e: bool,
    template: &Matcher,
    options: MatchOptions,
) -> anyhow::Result<SearchResult> {
    print_inline!("Capture screen             ");
    let (point, current_screen) = screen_cap(point, is_5e)?;
    print_inline!("Marking area into Vec<bool>");
    let (buff, count) = process_area(&current_screen, template, options);
    if count < options.limit_x() * options.limit_y() {
        //log::debug!("Early exit");
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
        update_status!(pos1, pos2);
        move_mouse_click(
            pos1 as i32,
            pos2 as i32,
            TEST_MODE.load(std::sync::atomic::Ordering::Relaxed),
        )?;

        return Ok(true);
    }

    Ok(false)
}

fn sleep_until_exit(second: u64) -> bool {
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
    let options_5e = MatchOptions::new(force_distance, X_LIMIT_5E, Y_LIMIT_5E);

    loop {
        sys.refresh_all();

        match target_5e::check_need_handle(sys.processes()) {
            CheckResult::NeedProcess => {
                print_inline!("Match 5e     ");

                let ret = check_image_match(
                    config.e5().into(),
                    true,
                    &target_5e::MATCH_TEMPLATE,
                    options_5e,
                )?;
                if handle_target(ret)? {
                    sleep_until_exit!(config.interval().handle_success());
                    continue;
                }
            }
            CheckResult::NoNeedProcess => {
                print_inline!("User is playing     ");
                sleep_until_exit!(config.interval().e5_wait());
                continue;
            }
            CheckResult::Next => {}
        }

        match target_main::check_primary_exec(sys.processes())? {
            CheckResult::NeedProcess => {
                print_inline!("Match CS2     ");
                //log::debug!("Check cs main");
                let ret = check_image_match(
                    config.cs2().into(),
                    false,
                    &target_main::MATCH_TEMPLATE,
                    options,
                )?;
                if handle_target(ret)? {
                    sleep_until_exit!(config.interval().handle_success());
                    continue;
                }
            }
            CheckResult::NoNeedProcess => {
                print_inline!("Not searching              ");
                sleep_until_exit!(config.interval().cs2_wait());
                continue;
            }
            CheckResult::Next => {}
        }
        //log::debug!("Next tick");
        print_inline!("Sleep                      ");
        if !TEST_MODE.load(std::sync::atomic::Ordering::Relaxed) {
            sleep_until_exit!(config.interval().each());
        } else {
            sleep_until_exit!(2);
        }
    }
    log::info!("User exit");
    Ok(())
}

fn real_main_guarder(config: &String, force_distance: bool) -> anyhow::Result<()> {
    log::info!("Started checking");
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
        #[cfg(not(feature = "gui"))]
        EXIT_SIGNAL
            .set(true)
            .unwrap_or_else(|_| std::process::exit(1));
        #[cfg(feature = "gui")]
        {
            eprintln!("Emergency exit: should performance exit via click close button");
            std::process::exit(1);
        }
    })
    .unwrap();
    let matches = clap::command!()
        .args(&[
            arg!([CONFIG] "Configure file").default_value("config.toml"),
            arg!(-n --"dry-run" "Dry run (do not click)"),
            arg!(--"save-image" "Save image each time take"),
            arg!(--"force-distance" "Use distance algorithm to check image"),
        ])
        .subcommands([
            Command::new("mouse").about("Display current mouse position"),
            Command::new("get-color")
                .about("Get RGB list from image file")
                .args(&[
                    arg!(<FILE> ... "Image file"),
                    arg!(--output <OUTPUT> "Output file").default_missing_value("output.rs"),
                ]),
            Command::new("distance")
                .about("Find best color for image file")
                .args(&[
                    arg!(<FILE> ... "RGB file, generate by get-color command"),
                    arg!(-d --direct "Process file direct as image"),
                    arg!(--"read-only" "No write, just read"),
                    arg!(--output <output> "Output result to file").default_value("output.txt"),
                ])
                .hide(cfg!(feature = "distance")),
            Command::new("test")
                .about("Test image is match specify matcher")
                .args(&[arg!(<FILE> "Test image"), arg!(--"5e" "Enable 5e match")]),
            Command::new("match")
                .about("Help subcommand for debug matcher")
                .args(&[arg!(<function> "Functions to match")
                    .value_parser([PossibleValue::new("cs2-lobby")])])
                .subcommands(&[
                    Command::new("screen").about("From screen").args(&[
                        arg!([interval] "Fetch interval(ms)")
                            .default_value("250")
                            .value_parser(clap::value_parser!(u64)),
                        arg!(--save <failed_only> "Save image")
                            .default_value("false")
                            .value_parser(clap::value_parser!(bool)),
                    ]),
                    Command::new("dir")
                        .alias("directory")
                        .about("about")
                        .args(&[
                            arg!(<directory> "Directory to check"),
                            arg!(--"fail-only" "Display failed only"),
                        ])
                        .hide(cfg!(not(feature = "jpeg"))),
                ])
                .subcommand_required(true),
            Command::new("gui"),
        ])
        .get_matches();

    if matches.get_flag("dry-run") {
        TEST_MODE.store(true, std::sync::atomic::Ordering::Relaxed);
        log::debug!("Dry running");
    }

    SAVE_IMAGE.store(
        matches.get_flag("save-image"),
        std::sync::atomic::Ordering::Relaxed,
    );
    let force_distance = matches.get_flag("force-distance");
    match matches.subcommand() {
        Some(("mouse", _)) => display_mouse(),
        Some(("get-color", matches)) => load_and_display(
            &matches.get_many::<String>("FILE").unwrap(),
            matches.get_one("output"),
        ),
        Some(("distance", matches)) => distance::calc_color_distance(
            matches.get_many::<String>("FILE").unwrap(),
            matches.get_one::<String>("output").unwrap(),
            matches.get_flag("read-only"),
            !matches.get_flag("direct"),
        ),
        Some(("test", matches)) => test_image(
            matches.get_one("FILE").unwrap(),
            matches.get_flag("5e"),
            force_distance,
        ),
        Some(("match", matches)) => {
            let function = matches.get_one::<String>("function").unwrap();
            match matches.subcommand() {
                Some(("screen", matches)) => continue_test_area(
                    function,
                    force_distance,
                    matches.get_flag("save"),
                    *matches.get_one("save").unwrap(),
                    *matches.get_one("interval").unwrap(),
                ),
                Some(("dir", matches)) => dir_match::test_files(
                    function,
                    matches.get_one::<String>("directory").unwrap(),
                    matches.get_flag("fail-only"),
                ),
                _ => unreachable!(),
            }
        }
        Some(("gui", _)) => gui::gui_entry(matches.get_one("CONFIG").unwrap(), force_distance),
        _ => real_main_guarder(matches.get_one("CONFIG").unwrap(), force_distance),
    }
}
