use crate::{BasicImageType, tools::RGB2};

pub(crate) struct Matcher {
    use_diff: bool,
    template: &'static [BasicImageType],
    threshold: f32,
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
        }
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

                    match match_algorithm(Default::default(), &buff, image.dimensions(), opts) {
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
