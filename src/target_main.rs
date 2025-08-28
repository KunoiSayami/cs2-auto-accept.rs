use std::collections::HashMap;

use image::Rgb;
use sysinfo::{Pid, Process};

use crate::{CheckResult, definitions::PROCESS_NAME, matcher::Matcher};

pub(crate) const MATCH_TEMPLATE: Matcher =
    Matcher::new(true, &[Rgb([52, 182, 81]), Rgb([58, 198, 90])], 20.0);

#[must_use]
pub(crate) fn check_primary_exec(process: &HashMap<Pid, Process>) -> CheckResult {
    if process.values().any(|x| x.name().eq(PROCESS_NAME)) {
        return CheckResult::NeedProcess;
    }
    CheckResult::Next
}
