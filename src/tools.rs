use std::collections::HashSet;

use clap::parser::ValuesRef;
use image::Rgb;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct RGB2 {
    r: u8,
    g: u8,
    b: u8,
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

pub fn load_and_display(p: &ValuesRef<String>) -> anyhow::Result<()> {
    let mut set = HashSet::new();
    println!("&[");

    for file in p.clone().into_iter() {
        let image = image::ImageReader::open(file)?.decode()?.into_rgb8();

        for (_, _, pixel) in image.enumerate_pixels() {
            let p = RGB2::from(pixel);

            set.insert(p);
            //println!("{},{},{}", pixel.0[0], pixel.0[1], pixel.0[2]);
        }

        for x in set.iter() {
            println!("Rgb([{x}]),")
        }
    }
    println!("]");

    Ok(())
}
