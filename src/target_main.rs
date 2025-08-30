use std::collections::HashMap;

use image::Rgb;
use sysinfo::{Pid, Process};

use crate::{CheckResult, definitions::PROCESS_NAME, matcher::Matcher, types::MatchOptions};

pub(crate) const MATCH_TEMPLATE: Matcher =
    Matcher::new(true, &[Rgb([52, 182, 81]), Rgb([58, 198, 90])], 20.0);
pub(crate) const LOBBY_MATCH_TEMPLATE: Matcher = Matcher::new(
    true,
    &[Rgb([11, 85, 10]), Rgb([4, 90, 4]), Rgb([9, 50, 7])],
    30.0,
);

pub(crate) fn check_primary_exec(process: &HashMap<Pid, Process>) -> anyhow::Result<CheckResult> {
    if process.values().any(|x| x.name().eq(PROCESS_NAME)) {
        //log::debug!("Check cs2 lobby");
        let ret = match crate::check_image_match(
            crate::PointOption::Transform(crate::tools::get_right_upon_side),
            false,
            &LOBBY_MATCH_TEMPLATE,
            MatchOptions::new(false, 4, 4),
        )? {
            crate::SearchResult::Found(_, _) => CheckResult::NeedProcess,
            crate::SearchResult::NotFound => CheckResult::NoNeedProcess,
        };
        return Ok(ret);
    }

    Ok(CheckResult::Next)
}
