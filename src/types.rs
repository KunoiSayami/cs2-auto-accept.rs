use serde::Deserialize;
use xcap::Monitor;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub struct Point {
    pos1x: i32,
    pos1y: i32,
    pos2x: i32,
    pos2y: i32,
}

impl Point {
    pub fn new(pos1x: i32, pos1y: i32, pos2x: i32, pos2y: i32) -> Self {
        Self {
            pos1x,
            pos1y,
            pos2x,
            pos2y,
        }
    }

    pub const fn height(&self) -> i32 {
        self.pos2y - self.pos1y
    }

    pub const fn width(&self) -> i32 {
        self.pos2x - self.pos1x
    }

    pub const fn x(&self) -> i32 {
        self.pos1x
    }

    pub const fn y(&self) -> i32 {
        self.pos1y
    }
}

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

pub(crate) enum PointOption {
    Some(Point),
    Transform(fn(Monitor) -> Point),
    None,
}

impl From<Option<Point>> for PointOption {
    fn from(value: Option<Point>) -> Self {
        match value {
            Some(p) => Self::Some(p),
            None => Self::None,
        }
    }
}

impl From<fn(Monitor) -> Point> for PointOption {
    fn from(value: fn(Monitor) -> Point) -> Self {
        Self::Transform(value)
    }
}
