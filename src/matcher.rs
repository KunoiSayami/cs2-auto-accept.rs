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
