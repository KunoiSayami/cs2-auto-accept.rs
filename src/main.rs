mod configure;
mod definitions;
mod target_5e;
mod target_main;

use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use clap::Command;
use configure::Point;
use enigo::{Enigo, Mouse};
use image::{DynamicImage, ImageBuffer, Rgb};
use sysinfo::{ProcessRefreshKind, RefreshKind};
use xcap::Monitor;

use crate::target_main::handle_target;

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

type SubImageType = Vec<u8>;
type ImageType = ImageBuffer<image::Rgb<u8>, SubImageType>;

fn determine_point(monitor: Monitor, is_5e: bool) -> anyhow::Result<Point> {
    let x = monitor.x()?;
    let y = monitor.y()? - 100;
    let height = monitor.height()? as i32;
    let width = monitor.width()? as i32;
    let h = if is_5e { 400 } else { 200 } / 2;
    let w = if is_5e { 800 } else { 400 } / 2;

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
        log::debug!("{real_point:?}");

        //return Ok(DynamicImage::from(image).into_rgb8());
        log::trace!("elapsed: {:?}", start.elapsed());
        //image.save("output.png")?;
        return Ok((real_point, DynamicImage::from(image).into_rgb8()));
    }
    Err(anyhow::anyhow!("Not found primary monitor"))
}

fn match_algorithm(point: Point, area: &ImageType, template: &[Rgb<u8>]) -> SearchResult {
    const X_LIMIT: usize = 10;
    const Y_LIMIT: usize = 8;
    let (pic_x, pic_y) = area.dimensions();
    let mut buff = vec![vec![false; pic_y as usize]; pic_x as usize];
    //let mut result = vec![vec![false; pic_y as usize - Y_LIMIT]; pic_x as usize - X_LIMIT];
    for (x, y, pixel) in area.enumerate_pixels() {
        buff[x as usize][y as usize] = template.iter().any(|x| x == pixel);
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
            'outer: for x in original_x..(original_x + X_LIMIT) {
                for y in original_y..(original_y + Y_LIMIT) {
                    if !buff[x][y] {
                        r = false;
                        break 'outer;
                    }
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
    template: &[Rgb<u8>],
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
            log::debug!("x: {x}, y: {y}");
            prev_x = x;
            prev_y = y;
        }
        sleep(Duration::from_millis(100));
    }
}

fn main() -> anyhow::Result<()> {
    let matches = clap::command!()
        .subcommand(Command::new("mouse"))
        .get_matches();

    match matches.subcommand() {
        Some(("mouse", _)) => {
            return display_mouse();
        }
        _ => {}
    }

    env_logger::Builder::from_default_env().init();

    let mut sys = sysinfo::System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );

    sys.refresh_all();

    handle_target()?;
    Ok(())
}
