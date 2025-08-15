use std::{collections::HashMap, sync::OnceLock};

use sysinfo::{Pid, Process};

use crate::{
    CheckResult, SubImageType, definitions::PROCESS_NAME, find_match_sub_image, load_image,
};

static SUB_IMAGE: OnceLock<SubImageType> = OnceLock::new();

#[must_use]
pub(crate) fn check_primary_exec(process: &HashMap<Pid, Process>) -> CheckResult {
    if process.values().any(|x| x.name().eq(PROCESS_NAME)) {
        return CheckResult::NeedProcess;
    }
    return CheckResult::Next;
}

pub(crate) fn handle_target() -> anyhow::Result<bool> {
    if SUB_IMAGE.get().is_none() {
        let image = load_image("cs2.png")?;
        SUB_IMAGE.set(image.into_raw()).unwrap();
    }
    let sub_image = SUB_IMAGE.get().unwrap();

    if let Some((pos1, pos2, trust_factor)) = find_match_sub_image(sub_image)? {
        log::trace!("{pos1} {pos2} {trust_factor:.2}");
    }

    Ok(false)
}
