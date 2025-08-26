#[cfg(target_os = "linux")]
mod linux {
    pub(crate) const PROCESS_NAME: &str = "cs2";
}

mod windows {
    pub(crate) const PROCESS_NAME: &str = "cs2.exe";
    pub(crate) const PROCESS_5E_NAME: &str = "5EClient.exe";
    pub(crate) const PROCESS_5E_ANTI_CHEAT_NAME: &str = "Bucky64.exe";
}

#[cfg(target_os = "linux")]
pub(crate) use linux::*;
#[cfg(target_os = "windows")]
pub(crate) use windows::*;
