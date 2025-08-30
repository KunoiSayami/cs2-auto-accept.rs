use std::{
    collections::HashSet,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    sync::{
        Arc,
        mpsc::{Receiver, channel},
    },
    thread::spawn,
};

use clap::parser::ValuesRef;

use crate::tools::RGB2;

#[derive(Clone, Copy, Debug, Default)]
struct RGB2Info {
    inner: RGB2,
    max: f32,
    min: f32,
    middle: f32,
    middle_10: f32,
    middle_n10: f32,
}

/* impl RGB2Info {
    fn diff(&self) -> f32 {
        self.max - self.min
    }
} */

impl std::fmt::Display for RGB2Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}, {}, {}",
            self.inner, self.max, self.min, self.middle, self.middle_10, self.middle_n10
        )
    }
}

enum DataEvent {
    New(RGB2Info),
    Close,
}

fn only_read() -> anyhow::Result<()> {
    let file = File::open("output.txt")?;
    let reader = BufReader::new(file);
    let lines = reader.lines();
    let mut v = vec![];

    for line in lines {
        let line = line?;
        let l = line.split(", ").collect::<Vec<&str>>();
        //println!("{l:?}");
        v.push(RGB2Info {
            inner: RGB2::new(l[0].parse()?, l[1].parse()?, l[2].parse()?),
            max: l[3].parse()?,
            min: l[4].parse()?,
            middle: l[5].parse()?,
            middle_10: l[6].parse()?,
            middle_n10: l[7].parse()?,
        });
    }
    v.sort_by(|x, y| x.middle.partial_cmp(&y.middle).unwrap());
    println!("R G B Max Min Middle Middle+10 Middle-10");
    for x in &v[..10] {
        println!("{x} {}", (x.middle_10 - x.middle_n10).abs());
    }
    //println!("{:?}", &v[..10]);
    Ok(())
}

fn write_thread(mut file: File, recv: Receiver<DataEvent>) -> anyhow::Result<()> {
    let mut v = Vec::with_capacity(11);

    while let Ok(event) = recv.recv() {
        match event {
            DataEvent::New(rgb2_info) => {
                /* if rgb2_info.diff() < diff && max > rgb2_info.max {
                    basic = rgb2_info;
                    diff = rgb2_info.diff();
                    max = rgb2_info.max;
                } */
                let mut s = rgb2_info.to_string();
                s.push('\n');
                file.write_all(s.as_bytes())?;
                v.push(rgb2_info);
                v.sort_unstable_by(|v, other| v.middle.partial_cmp(&other.middle).unwrap());
                if v.len() > 10 {
                    v.pop();
                }
            }
            DataEvent::Close => break,
        }
    }
    for x in &v {
        println!("{x} {}", (x.middle_10 - x.middle_n10).abs());
    }
    Ok(())
}

fn load_image(file: &str) -> anyhow::Result<Vec<RGB2>> {
    let mut set = HashSet::new();
    let image = image::ImageReader::open(file)?.decode()?.into_rgb8();

    for (_, _, pixel) in image.enumerate_pixels() {
        let p = RGB2::from(pixel);

        set.insert(p);
        //println!("{},{},{}", pixel.0[0], pixel.0[1], pixel.0[2]);
    }

    Ok(set.into_iter().collect())
}

fn load_rgb(file: &str, is_txt: bool) -> anyhow::Result<Vec<RGB2>> {
    if is_txt {
        return load_image(file);
    }
    let reader = BufReader::new(File::open(file)?);
    let lines = reader.lines();

    let mut v = vec![];

    for line in lines {
        let line = line?;
        let s = line.split(", ").collect::<Vec<_>>();
        v.push(RGB2::new(s[0].parse()?, s[1].parse()?, s[2].parse()?));
    }

    Ok(v)
}

pub(crate) fn calc_color_distance(
    files: ValuesRef<'_, String>,
    output: &str,
    read_only: bool,
    is_txt: bool,
) -> anyhow::Result<()> {
    if read_only {
        return only_read();
    }
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(output)?;
    let (s, r) = channel();
    let tr = spawn(move || write_thread(file, r));

    let pool = threadpool::Builder::new().build();

    let mut colors = Vec::new();

    for file in files.into_iter() {
        colors.append(&mut load_rgb(file, is_txt)?);
    }
    let input = Arc::new(colors);

    for r in 0..=128 {
        for g in 50..=255 {
            for b in 0..=128 {
                let basic = RGB2::new(r, g, b);
                let sender = s.clone();
                let input = input.clone();
                pool.execute(move || {
                    let ret = inner_calc_color_distance(basic, input);
                    sender.send(DataEvent::New(ret)).unwrap();
                });
            }
        }
    }
    pool.join();
    s.send(DataEvent::Close).ok();
    tr.join().unwrap()?;
    Ok(())
}

#[must_use]
fn inner_calc_color_distance(basic: RGB2, input: Arc<Vec<RGB2>>) -> RGB2Info {
    //let basic = RGB2::new(80, 255, 20);
    //let basic = RGB2::new(70, 255, 30);
    let mut v = Vec::new();

    let mut max = f32::MIN;
    let mut min = f32::MAX;
    for other in input.iter() {
        //let other = RGB2::from(x);
        let d = basic.distance(other);
        max = max.max(d);
        min = min.min(d);
        v.push(d);
        //println!("{d}");
    }

    //let instant = Instant::now();
    v.sort_by(|x, y| x.partial_cmp(y).unwrap());
    //log::debug!("elapsed: {:?}", instant.elapsed());

    RGB2Info {
        inner: basic,
        max,
        min,
        middle: v[v.len() / 2],
        middle_10: v[v.len() / 2 + 10],
        middle_n10: v[v.len() / 2 - 10],
    }
}
