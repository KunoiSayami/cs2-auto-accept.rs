#[cfg(not(windows))]
mod fallback;
#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
pub(crate) use fallback::*;
#[cfg(windows)]
pub(crate) use windows::*;
