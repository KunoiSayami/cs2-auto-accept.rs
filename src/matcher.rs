use std::sync::atomic::AtomicBool;

use crate::{BasicImageType, tools::RGB2};

pub(crate) static OVERRIDE_USE_DISTANCE: AtomicBool = AtomicBool::new(false);

#[cfg(not(debug_assertions))]
#[inline(always)]
fn override_use() -> bool {
    false
}

#[cfg(debug_assertions)]
#[inline(always)]
fn override_use() -> bool {
    OVERRIDE_USE_DISTANCE.load(std::sync::atomic::Ordering::Relaxed)
}

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

    pub(crate) fn use_diff(&self) -> bool {
        self.use_diff
    }

    pub(crate) fn check(&self, pixel: &BasicImageType) -> bool {
        if !self.use_diff && !override_use() {
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
