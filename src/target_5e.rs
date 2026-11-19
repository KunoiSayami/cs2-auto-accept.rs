use image::Rgb;
use std::collections::HashMap;
use sysinfo::{Pid, Process};

use crate::{
    CheckResult,
    definitions::{PROCESS_5E_NAME, PROCESS_NAME},
    matcher::{Matcher, Shape},
};

#[cfg(windows)]
pub(crate) fn is_5e_foreground(title: &str) -> bool {
    use winsafe::HWND;
    HWND::GetForegroundWindow()
        .and_then(|hwnd| hwnd.GetWindowText().ok())
        .map(|t| t.contains(title))
        .unwrap_or(false)
}

#[cfg(not(windows))]
pub(crate) fn is_5e_foreground() -> bool {
    false
}

/// The ready button measured ~283x59 (aspect ~4.8, fill ~0.94) at 100% scaling.
/// Size limits stay loose so other scaling still passes; the green bar on the
/// post-match screen (400x12) and the glow band (fill < 0.4) are rejected.
pub const MATCH_TEMPLATE: Matcher = Matcher::new(true, &[Rgb([72, 180, 30])], 90.0)
    .with_shape(Shape::new(140, 30, (3.0, 8.0), 0.8));

pub(crate) fn check_need_handle(process: &HashMap<Pid, Process>) -> CheckResult {
    let process_found = process.values().any(|x| x.name().eq(PROCESS_5E_NAME));

    if process_found {
        let cs_found = process.values().any(|x| x.name().eq(PROCESS_NAME));
        if cs_found {
            return CheckResult::NoNeedProcess;
        }
        return CheckResult::NeedProcess;
    }
    CheckResult::Next
}
