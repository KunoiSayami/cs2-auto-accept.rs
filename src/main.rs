mod definitions;
mod target_5e;
mod target_main;

use std::time::Instant;

use find_subimage::{Backend, SubImageFinderState};
use image::{DynamicImage, ImageBuffer};
use sysinfo::{ProcessRefreshKind, RefreshKind};
use xcap::Monitor;

use crate::target_main::handle_target;

enum CheckResult {
    NeedProcess,
    // Wait for another check
    NoNeedProcess,
    // Call next check (if there is)
    Next,
}

type SubImageType = Vec<u8>;
type ImageType = ImageBuffer<image::Rgb<u8>, SubImageType>;

fn screen_cap() -> anyhow::Result<ImageType> {
    let start = Instant::now();
    let monitors = Monitor::all().unwrap();

    for monitor in monitors {
        if !monitor.is_primary().unwrap_or_default() {
            continue;
        }
        let image = monitor.capture_image().unwrap();

        //return Ok(DynamicImage::from(image).into_rgb8());
        return Ok(DynamicImage::from(image).into_rgb8());
    }
    log::trace!("elapsed: {:?}", start.elapsed());
    Err(anyhow::anyhow!("Not found primary monitor"))
}

fn load_image(p: &str) -> anyhow::Result<ImageType> {
    let image = image::ImageReader::open(p)?.decode()?.into_rgb8();
    Ok(image)
}

fn find_match_sub_image(sub_image: &SubImageType) -> anyhow::Result<Option<(usize, usize, f32)>> {
    let image = screen_cap()?;

    let (image1_w, image1_h) = image.dimensions();

    let (image2_w, image2_h) = image.dimensions();
    let mut finder = SubImageFinderState::new();
    let raw = image.into_raw();
    log::trace!("{image1_w} {image1_h} {}", raw.len() / 1920 / 1080);

    let pos = finder.find_subimage_positions_with_backend(
        (sub_image, image2_w as usize, image2_h as usize),
        (&raw, image1_w as usize, image1_h as usize),
        &Backend::OpenCV { threshold: 1.0 },
        3,
    );
    log::debug!("{pos:?}");

    Ok(pos.get(0).copied())
}

fn main() -> anyhow::Result<()> {
    clap::command!().get_matches();

    env_logger::Builder::from_default_env().init();

    let mut sys = sysinfo::System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );

    sys.refresh_all();

    handle_target()?;
    Ok(())
}
