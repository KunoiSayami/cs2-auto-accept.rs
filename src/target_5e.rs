mod target {
    use std::collections::HashMap;
    use sysinfo::{Pid, Process};

    use crate::{
        CheckResult,
        definitions::{PROCESS_5E_ANTI_CHEAT_NAME, PROCESS_5E_NAME},
        screen_cap,
    };

    #[must_use]
    pub(crate) fn check_need_handle(process: &HashMap<Pid, Process>) -> CheckResult {
        let anti_cheat_found = process
            .values()
            .any(|x| x.name().eq(PROCESS_5E_ANTI_CHEAT_NAME));
        let process_found = process.values().any(|x| x.name().eq(PROCESS_5E_NAME));

        if process_found {
            if anti_cheat_found {
                return CheckResult::NoNeedProcess;
            }
            return CheckResult::NeedProcess;
        }
        return CheckResult::Next;
    }

    pub(crate) fn handle_target() -> anyhow::Result<bool> {
        /* let image = screen_cap()?; */
        Ok(false)
    }

    fn click() -> anyhow::Result<()> {
        Ok(())
    }
}

mod un_target {}

pub(crate) use target::*;
