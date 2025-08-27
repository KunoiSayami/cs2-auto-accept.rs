mod configure;
mod definitions;
mod distance;
mod matcher;
mod target_5e;
mod target_main;
mod tools;

use std::{
    sync::atomic::AtomicBool,
    thread::sleep,
    time::{Duration, Instant},
};

use clap::{Command, arg};
use configure::{Configure, Point};
use enigo::{Enigo, Mouse};
use image::{DynamicImage, ImageBuffer, Rgb};
use sysinfo::{ProcessRefreshKind, RefreshKind};
use tools::load_and_display;
use xcap::Monitor;

use crate::{distance::calc_color_distance, matcher::Matcher};

static TEST_MODE: AtomicBool = AtomicBool::new(false);

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
        if TEST_MODE.load(std::sync::atomic::Ordering::Relaxed) {
            image.save("output.png")?;
        }
        return Ok((real_point, DynamicImage::from(image).into_rgb8()));
    }
    Err(anyhow::anyhow!("Not found primary monitor"))
}

fn match_algorithm(point: Point, area: &ImageType, template: &Matcher) -> SearchResult {
    const X_LIMIT: usize = 10;
    const Y_LIMIT: usize = 8;
    let (pic_x, pic_y) = area.dimensions();
    let mut buff = vec![vec![false; pic_y as usize]; pic_x as usize];
    //let mut result = vec![vec![false; pic_y as usize - Y_LIMIT]; pic_x as usize - X_LIMIT];
    for (x, y, pixel) in area.enumerate_pixels() {
        buff[x as usize][y as usize] = template.check(pixel);
    }

    let x_start = X_LIMIT / 2;
    let x_end = pic_x as usize - x_start;
    let y_start = Y_LIMIT / 2;
    let y_end = pic_y as usize - y_start;

    for x in x_start..x_end as usize {
        for y in y_start..y_end as usize {
            let original_x = x - x_start;
            let original_y = y - y_start;
            let mut r = true;
            //let mut matched = 0;
            'outer: for x in original_x..(original_x + X_LIMIT) {
                for y in original_y..(original_y + Y_LIMIT) {
                    if !buff[x][y] {
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
    return SearchResult::NotFound;
}

pub(crate) fn check_image_match(
    point: Option<Point>,
    is_5e: bool,
    template: &Matcher,
) -> anyhow::Result<SearchResult> {
    let (point, current_screen) = screen_cap(point, is_5e)?;
    Ok(match_algorithm(point, &current_screen, template))
}

fn display_mouse() -> anyhow::Result<()> {
    let eg = enigo::Enigo::new(&enigo::Settings::default())?;
    let mut prev_x = 0;
    let mut prev_y = 0;

    loop {
        let (x, y) = eg.location()?;
        if prev_x != x || prev_y != y {
            println!("x: {x}, y: {y}");
            prev_x = x;
            prev_y = y;
        }
        sleep(Duration::from_millis(100));
    }
}

fn handle_target(point: Option<Point>, template: &Matcher, eg: &mut Enigo) -> anyhow::Result<bool> {
    let test_mode = TEST_MODE.load(std::sync::atomic::Ordering::Relaxed);
    if let SearchResult::Found(pos1, pos2) =
        check_image_match(point, template.use_diff(), template)?
    {
        log::debug!("x: {pos1}, y: {pos2}");
        eg.move_mouse(pos1 as i32, pos2 as i32, enigo::Coordinate::Abs)?;
        if !test_mode {
            eg.button(enigo::Button::Left, enigo::Direction::Click)?;
            sleep(Duration::from_secs(1));
            eg.button(enigo::Button::Left, enigo::Direction::Click)?;
        } else {
            log::debug!("clicked");
        }
        return Ok(true);
    }

    Ok(false)
}

fn real_main(config: &String) -> anyhow::Result<()> {
    let config = Configure::load(config).unwrap_or_default();
    let mut sys = sysinfo::System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );
    let mut eg = Enigo::new(&enigo::Settings::default())?;

    log::info!("Starting listening");

    loop {
        sys.refresh_all();

        match target_5e::check_need_handle(sys.processes()) {
            CheckResult::NeedProcess => {
                handle_target(config.e5(), &target_5e::MATCH_TEMPLATE, &mut eg)?;
            }
            CheckResult::NoNeedProcess => {
                sleep(Duration::from_secs(60));
                continue;
            }
            CheckResult::Next => {}
        }

        match target_main::check_primary_exec(sys.processes()) {
            CheckResult::NeedProcess => {
                handle_target(config.cs2(), &target_main::MATCH_TEMPLATE, &mut eg)?;
            }
            CheckResult::NoNeedProcess => {
                unimplemented!()
            }
            CheckResult::Next => {}
        }
        //log::debug!("Next tick");
        if !TEST_MODE.load(std::sync::atomic::Ordering::Relaxed) {
            sleep(Duration::from_secs(5));
        } else {
            sleep(Duration::from_millis(300));
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();
    let matches = clap::command!()
        .args(&[
            arg!([CONFIG] "Configure file").default_value("config.toml"),
            arg!(--"dry-run" "Dry run (do not click)"),
        ])
        .subcommand(Command::new("mouse"))
        .subcommand(Command::new("get-color").args(&[
            arg!(<FILE> ... "Image file"),
            arg!(--output <OUTPUT> "Output file").default_missing_value("output.rs"),
        ]))
        .subcommand(Command::new("distance").args(&[
            arg!(<FILE> "RGB file, generate by get-color command"),
            arg!(--"read-only" "No write, just read"),
        ]))
        .get_matches();

    TEST_MODE.store(
        matches.get_flag("dry-run"),
        std::sync::atomic::Ordering::Relaxed,
    );

    match matches.subcommand() {
        Some(("mouse", _)) => display_mouse(),
        Some(("get-color", matches)) => load_and_display(
            &matches.get_many("FILE").unwrap(),
            matches.get_one("output"),
        ),
        Some(("distance", matches)) => calc_color_distance(
            matches.get_one("FILE").unwrap(),
            matches.get_flag("read-only"),
        ),
        _ => real_main(matches.get_one("CONFIG").unwrap()),
    }
}
