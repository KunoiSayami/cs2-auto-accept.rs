use std::fs::read_to_string;

use serde::Deserialize;

use crate::types::Point;

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Interval {
    #[serde(alias = "after-success", alias = "handle-success")]
    handle_success: u64,
    #[serde(alias = "loop")]
    each: u64,
    #[serde(alias = "long-sleep")]
    long: u64,
    #[serde(alias = "cs2-wait", alias = "cs-wait")]
    cs2_wait: u64,
    #[serde(rename = "5e-wait", alias = "5e_wait")]
    e5_wait: u64,
}

impl Interval {
    pub fn handle_success(&self) -> u64 {
        self.handle_success
    }

    pub fn each(&self) -> u64 {
        self.each
    }

    pub fn cs2_wait(&self) -> u64 {
        self.cs2_wait
    }

    pub fn e5_wait(&self) -> u64 {
        self.e5_wait
    }

    pub fn long(&self) -> u64 {
        self.long
    }
}

impl Default for Interval {
    fn default() -> Self {
        Self {
            handle_success: 2,
            each: 3,
            cs2_wait: 16,
            e5_wait: 20,
            long: 10,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Default)]
pub struct Configure {
    cs2: Option<Point>,
    #[serde(rename = "5e")]
    e5: Option<Point>,
    #[serde(default)]
    interval: Interval,
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

    pub fn interval(&self) -> Interval {
        self.interval
    }
}
