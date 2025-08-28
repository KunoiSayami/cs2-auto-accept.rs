use std::{collections::HashSet, fs::OpenOptions, io::Write};

use chrono::Local;
use clap::parser::ValuesRef;
use image::Rgb;

use crate::{
    X_LIMIT, Y_LIMIT, configure::Point, match_algorithm, process_area, types::MatchOptions,
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
pub(crate) struct RGB2 {
    r: u8,
    g: u8,
    b: u8,
}

impl RGB2 {
    #[cfg(feature = "distance")]
    pub(crate) fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub(crate) fn distance(&self, other: &Self) -> f32 {
        let r = self.r as i32 - other.r as i32;
        let g = self.g as i32 - other.g as i32;
        let b = self.b as i32 - other.b as i32;
        let d2 = r * r + g * g + b * b;
        (d2 as f32).sqrt()
    }
}

impl From<Rgb<u8>> for RGB2 {
    fn from(value: Rgb<u8>) -> Self {
        Self {
            r: value.0[0],
            g: value.0[1],
            b: value.0[2],
        }
    }
}

impl From<&Rgb<u8>> for RGB2 {
    fn from(value: &Rgb<u8>) -> Self {
        Self {
            r: value.0[0],
            g: value.0[1],
            b: value.0[2],
        }
    }
}

impl std::fmt::Display for RGB2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}, {}", self.r, self.g, self.b)
    }
}

pub fn load_and_display(p: &ValuesRef<String>, output_file: Option<&String>) -> anyhow::Result<()> {
    let mut set = HashSet::new();
    let mut output = vec![];

    for file in p.clone() {
        let image = image::ImageReader::open(file)?.decode()?.into_rgb8();

        for (_, _, pixel) in image.enumerate_pixels() {
            let p = RGB2::from(pixel);

            set.insert(p);
            //println!("{},{},{}", pixel.0[0], pixel.0[1], pixel.0[2]);
        }

        for x in set.iter() {
            output.push(x.to_string());
        }
    }
    //output.push("]".into());

    let data = output.join("\n");

    if let Some(output_file) = output_file {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(output_file)?;
        file.write_all(data.as_bytes())?;
    } else {
        println!("{data}");
    }
    Ok(())
}

pub(crate) fn timestamp_fmt(fmt: &str) -> String {
    Local::now().format(fmt).to_string()
}

pub(crate) fn test_image(file: &String, is_5e: bool, force_distance: bool) -> anyhow::Result<()> {
    let image = image::ImageReader::open(file)?.decode()?.into_rgb8();

    let opts = MatchOptions::new(force_distance, crate::X_LIMIT, crate::Y_LIMIT);

    let (buff, count) = process_area(
        &image,
        if is_5e {
            &crate::target_5e::MATCH_TEMPLATE
        } else {
            &crate::target_main::MATCH_TEMPLATE
        },
        opts,
    );
    if count < X_LIMIT * Y_LIMIT {
        println!("Early exit {count}");
        return Ok(());
    }
    let (x, y) = image.dimensions();
    let ret = match_algorithm(Point::new(0, 0, x as i32, y as i32), &buff, (x, y), opts);
    println!("{ret:?}");
    Ok(())
}
