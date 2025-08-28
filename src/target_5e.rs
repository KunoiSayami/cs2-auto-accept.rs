use image::Rgb;
use std::collections::HashMap;
use sysinfo::{Pid, Process};

use crate::{
    CheckResult,
    definitions::{PROCESS_5E_NAME, PROCESS_NAME},
    matcher::Matcher,
};

pub const MATCH_TEMPLATE: Matcher = Matcher::new(true, &[Rgb([72, 180, 30])], 90.0);

#[must_use]
pub(crate) fn check_need_handle(process: &HashMap<Pid, Process>) -> CheckResult {
    let cs_found = process.values().any(|x| x.name().eq(PROCESS_NAME));
    let process_found = process.values().any(|x| x.name().eq(PROCESS_5E_NAME));

    if process_found {
        if cs_found {
            return CheckResult::NoNeedProcess;
        }
        return CheckResult::NeedProcess;
    }
    CheckResult::Next
}
