use std::fs::read_to_string;

use serde::Deserialize;

use crate::types::Point;

fn default_long_sleep() -> u64 {
    10
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Interval {
    #[serde(alias = "after-success", alias = "handle-success")]
    handle_success: u64,
    #[serde(alias = "loop")]
    each: u64,
    #[serde(alias = "long-sleep", default = "default_long_sleep")]
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

#[cfg(feature = "obs")]
#[derive(Clone, Debug, Deserialize)]
pub struct ObsIntegration {
    #[serde(default)]
    enabled: bool,
    #[serde(default = "ObsIntegration::default_host")]
    host: String,
    #[serde(default = "ObsIntegration::default_port")]
    port: u16,
    password: Option<String>,
}

#[cfg(feature = "obs")]
impl ObsIntegration {
    fn default_host() -> String {
        "127.0.0.1".to_string()
    }

    fn default_port() -> u16 {
        4455
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }
}

#[cfg(feature = "obs")]
impl Default for ObsIntegration {
    fn default() -> Self {
        Self {
            enabled: false,
            host: Self::default_host(),
            port: Self::default_port(),
            password: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Configure {
    cs2: Option<Point>,
    #[serde(rename = "5e")]
    e5: Option<Point>,
    #[serde(default)]
    interval: Interval,
    #[cfg(feature = "obs")]
    #[serde(default)]
    obs: ObsIntegration,
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

    #[cfg(feature = "obs")]
    pub fn obs(&self) -> &ObsIntegration {
        &self.obs
    }
}
