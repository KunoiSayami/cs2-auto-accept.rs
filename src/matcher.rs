use crate::{BasicImageType, tools::RGB2};

pub(crate) struct Matcher {
    use_diff: bool,
    template: &'static [BasicImageType],
}

impl Matcher {
    pub(crate) const fn new(use_diff: bool, template: &'static [BasicImageType]) -> Self {
        Self { use_diff, template }
    }

    pub(crate) fn use_diff(&self) -> bool {
        self.use_diff
    }

    pub(crate) fn check(&self, pixel: &BasicImageType) -> bool {
        if self.use_diff {
            return self.template.iter().any(|x| x == pixel);
        }
        let pixel = RGB2::from(pixel);
        pixel.distance(&RGB2::from(self.template[0])) < 90.0
    }
}
