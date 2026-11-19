use crate::{BasicImageType, tools::RGB2};

/// Connected area of matched pixels, as an inclusive bounding box.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Region {
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    count: usize,
}

impl Region {
    /// Flood-fill (4-connected) the matched area containing `(x, y)`,
    /// marking every visited pixel in `visited`.
    pub(crate) fn flood_fill(
        buff: &[Vec<bool>],
        x: usize,
        y: usize,
        visited: &mut [Vec<bool>],
    ) -> Self {
        let mut region = Self {
            x0: x,
            y0: y,
            x1: x,
            y1: y,
            count: 0,
        };
        let mut stack = vec![(x, y)];
        visited[x][y] = true;

        while let Some((x, y)) = stack.pop() {
            region.count += 1;
            region.x0 = region.x0.min(x);
            region.x1 = region.x1.max(x);
            region.y0 = region.y0.min(y);
            region.y1 = region.y1.max(y);

            let neighbors = [
                x.checked_sub(1).map(|x| (x, y)),
                (x + 1 < buff.len()).then_some((x + 1, y)),
                y.checked_sub(1).map(|y| (x, y)),
                (y + 1 < buff[x].len()).then_some((x, y + 1)),
            ];
            for (nx, ny) in neighbors.into_iter().flatten() {
                if buff[nx][ny] && !visited[nx][ny] {
                    visited[nx][ny] = true;
                    stack.push((nx, ny));
                }
            }
        }
        region
    }

    pub(crate) fn width(&self) -> usize {
        self.x1 - self.x0 + 1
    }

    pub(crate) fn height(&self) -> usize {
        self.y1 - self.y0 + 1
    }

    /// Ratio of matched pixels inside the bounding box.
    pub(crate) fn fill(&self) -> f32 {
        self.count as f32 / (self.width() * self.height()) as f32
    }

    pub(crate) fn center(&self) -> (usize, usize) {
        ((self.x0 + self.x1) / 2, (self.y0 + self.y1) / 2)
    }
}

/// Expected shape of the matched area (e.g. a button).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Shape {
    min_width: usize,
    min_height: usize,
    /// Allowed range of width / height
    aspect: (f32, f32),
    min_fill: f32,
}

impl Shape {
    pub(crate) const fn new(
        min_width: usize,
        min_height: usize,
        aspect: (f32, f32),
        min_fill: f32,
    ) -> Self {
        Self {
            min_width,
            min_height,
            aspect,
            min_fill,
        }
    }

    pub(crate) fn accepts(&self, region: &Region) -> bool {
        let aspect = region.width() as f32 / region.height() as f32;
        region.width() >= self.min_width
            && region.height() >= self.min_height
            && (self.aspect.0..=self.aspect.1).contains(&aspect)
            && region.fill() >= self.min_fill
    }
}

pub(crate) struct Matcher {
    use_diff: bool,
    template: &'static [BasicImageType],
    threshold: f32,
    shape: Option<Shape>,
}

impl Matcher {
    pub(crate) const fn new(
        use_diff: bool,
        template: &'static [BasicImageType],
        threshold: f32,
    ) -> Self {
        Self {
            use_diff,
            template,
            threshold,
            shape: None,
        }
    }

    pub(crate) const fn with_shape(mut self, shape: Shape) -> Self {
        self.shape = Some(shape);
        self
    }

    pub(crate) fn shape(&self) -> Option<&Shape> {
        self.shape.as_ref()
    }

    pub(crate) fn check(&self, pixel: &BasicImageType, force_distance: bool) -> bool {
        if !self.use_diff && !force_distance {
            //let ret = ;
            //println!("{pixel:?} {ret:?}");
            return self.template.iter().any(|x| x == pixel);
        }
        let pixel = RGB2::from(pixel);
        self.template
            .iter()
            .any(|x| pixel.distance(&RGB2::from(x)) < self.threshold)
    }
}

#[cfg(feature = "jpeg")]
pub(crate) mod dir_match {
    use std::{fs::DirEntry, sync::mpsc};

    use anyhow::anyhow;

    use crate::{match_algorithm, process_area, types::MatchOptions};

    fn recv_thread(display_fail_only: bool, recv: mpsc::Receiver<Option<(DirEntry, bool)>>) {
        let mut success = 0;
        let mut failed = 0;

        while let Ok(event) = recv.recv() {
            let Some((file, ret)) = event else {
                break;
            };
            if ret {
                success += 1;
            } else {
                failed += 1;
            }
            if display_fail_only && ret {
                continue;
            }
            println!("{:?} {ret}", file.file_name());
        }
        println!(
            "Success/Failed/Total: {success}/{failed}/{}",
            success + failed
        );
    }

    pub(crate) fn test_files(
        function: &str,
        directory: &str,
        display_fail_only: bool,
    ) -> anyhow::Result<()> {
        let pool = threadpool::Builder::new().build();

        let template = match function {
            "cs2-lobby" => &crate::target_main::LOBBY_MATCH_TEMPLATE,
            _ => unreachable!(),
        };
        let opts = match function {
            "cs2-lobby" => MatchOptions::new(false, 2, 2),
            _ => unreachable!(),
        };

        let (sender, r) = mpsc::channel();

        let files =
            std::fs::read_dir(directory).map_err(|e| anyhow!("List directory error: {e:?}"))?;

        let recv_handler = std::thread::spawn(move || recv_thread(display_fail_only, r));

        for file in files.into_iter().collect::<Result<Vec<_>, _>>()? {
            let sender = sender.clone();
            pool.execute(move || {
                (|| -> anyhow::Result<()> {
                    let image = image::ImageReader::open(file.path())?.decode()?.into_rgb8();
                    let (buff, count) = process_area(&image, template, opts);

                    if count < opts.limit_x() * opts.limit_y() {
                        sender.send(Some((file, false))).ok();
                        return Ok(());
                    }

                    match match_algorithm(
                        Default::default(),
                        &buff,
                        image.dimensions(),
                        opts,
                        template.shape(),
                    ) {
                        crate::SearchResult::Found(_, _) => sender.send(Some((file, true))).ok(),
                        crate::SearchResult::NotFound => sender.send(Some((file, false))).ok(),
                    };

                    Ok(())
                })()
                .inspect_err(|e| log::error!("{e:?}"))
                .ok();
            });
        }

        pool.join();
        sender.send(None).ok();
        recv_handler.join().unwrap();
        Ok(())
    }
}
