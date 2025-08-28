use std::fs::read_to_string;

use serde::Deserialize;

use crate::types::Point;

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
