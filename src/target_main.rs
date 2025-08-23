use std::collections::HashMap;

use enigo::Mouse;
use image::Rgb;
use sysinfo::{Pid, Process};

use crate::{CheckResult, SearchResult, check_image_match, definitions::PROCESS_NAME};

const MATCH_TEMPLATE: &[Rgb<u8>] = &[Rgb([52, 182, 81])];

#[must_use]
pub(crate) fn check_primary_exec(process: &HashMap<Pid, Process>) -> CheckResult {
    if process.values().any(|x| x.name().eq(PROCESS_NAME)) {
        return CheckResult::NeedProcess;
    }
    return CheckResult::Next;
}

pub(crate) fn handle_target() -> anyhow::Result<bool> {
    if let SearchResult::Found(pos1, pos2) = check_image_match(None, false, MATCH_TEMPLATE)? {
        log::trace!("{pos1} {pos2}");
        let mut eg = enigo::Enigo::new(&enigo::Settings::default())?;
        eg.move_mouse(pos1 as i32, pos2 as i32, enigo::Coordinate::Abs)?;
    }

    Ok(false)
}
