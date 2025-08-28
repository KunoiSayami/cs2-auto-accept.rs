use std::collections::HashMap;

use image::Rgb;
use sysinfo::{Pid, Process};

use crate::{
    CheckResult, check_image_match, definitions::PROCESS_NAME, matcher::Matcher,
    tools::get_right_upon_side,
};

pub(crate) const MATCH_TEMPLATE: Matcher =
    Matcher::new(true, &[Rgb([52, 182, 81]), Rgb([58, 198, 90])], 20.0);
pub(crate) const LOBBY_MATCH_TEMPLATE: Matcher = Matcher::new(true, &[Rgb([9, 128, 6])], 80.0);

#[must_use]
pub(crate) fn check_primary_exec(process: &HashMap<Pid, Process>) -> anyhow::Result<CheckResult> {
    if process.values().any(|x| x.name().eq(PROCESS_NAME)) {
        let ret = match check_image_match(
            crate::PointOption::Transform(get_right_upon_side),
            false,
            &LOBBY_MATCH_TEMPLATE,
            Default::default(),
        )? {
            crate::SearchResult::Found(_, _) => CheckResult::NeedProcess,
            crate::SearchResult::NotFound => CheckResult::NoNeedProcess,
        };
        return Ok(ret);
    }

    Ok(CheckResult::Next)
}
