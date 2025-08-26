use std::fs::read_to_string;

use serde::Deserialize;

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

#[derive(Clone, Copy, Debug, Deserialize, Default)]
pub struct Configure {
    cs2: Option<Point>,
    #[serde(rename = "5e")]
    e5: Option<Point>,
}

impl Configure {
    pub fn cs2(&self) -> Option<Point> {
        self.cs2
    }

    pub fn e5(&self) -> Option<Point> {
        self.e5
    }

    pub fn load(file: &String) -> anyhow::Result<Self> {
        Ok(toml::from_str(&read_to_string(file)?)?)
    }
}
