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

/// The "接受比赛" button is a dark purple vertical gradient, from (19, 5, 58)
/// at the top to (62, 28, 133) at the bottom, so three stops cover it. The
/// threshold is kept low because the surrounding background (~(30, 31, 35)) is
/// only ~37 away from the darkest stop.
/// It measured ~279x71 (aspect ~3.9, fill ~0.89) at 100% scaling. Size limits
/// stay loose so other scaling still passes; small purple blobs in the blurred
/// background (e.g. 52x44, fill 0.36) are rejected.
pub const MATCH_TEMPLATE: Matcher = Matcher::new(
    true,
    &[Rgb([20, 6, 60]), Rgb([36, 13, 94]), Rgb([55, 24, 125])],
    25.0,
)
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
