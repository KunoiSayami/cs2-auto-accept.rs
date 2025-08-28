#[derive(Clone, Copy, Debug)]
pub(crate) struct MatchOptions {
    force_distance: bool,
    limit_x: usize,
    limit_y: usize,
}

impl MatchOptions {
    pub(crate) fn new(force_distance: bool, limit_x: usize, limit_y: usize) -> Self {
        Self {
            force_distance,
            limit_x,
            limit_y,
        }
    }

    pub(crate) fn force_distance(&self) -> bool {
        self.force_distance
    }

    pub(crate) fn limit_x(&self) -> usize {
        self.limit_x
    }

    pub(crate) fn limit_y(&self) -> usize {
        self.limit_y
    }
}
