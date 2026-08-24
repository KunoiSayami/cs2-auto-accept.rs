use std::{thread::sleep, time::Duration};

use winsafe::{
    GetCursorPos, HwKbMouse, MOUSEINPUT, RegistryValue, SendInput, SetCursorPos, co::MOUSEEVENTF,
    co::REG_OPTION, co::RRF,
};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "CS2AutoAccept";

pub(crate) fn is_autostart_enabled() -> bool {
    winsafe::HKEY::CURRENT_USER
        .RegGetValue(Some(RUN_KEY), Some(VALUE_NAME), RRF::RT_REG_SZ)
        .is_ok()
}

pub(crate) fn set_autostart_enabled(enabled: bool) -> anyhow::Result<()> {
    if enabled {
        let exe_path = std::env::current_exe()?;
        let (hkey, _) = winsafe::HKEY::CURRENT_USER.RegCreateKeyEx(
            RUN_KEY,
            None,
            REG_OPTION::default(),
            winsafe::co::KEY::ALL_ACCESS,
            None,
        )?;
        hkey.RegSetValueEx(
            Some(VALUE_NAME),
            RegistryValue::Sz(format!("\"{}\"", exe_path.display())),
        )?;
    } else {
        let hkey = winsafe::HKEY::CURRENT_USER.RegOpenKeyEx(
            Some(RUN_KEY),
            REG_OPTION::default(),
            winsafe::co::KEY::ALL_ACCESS,
        )?;
        match hkey.RegDeleteValue(Some(VALUE_NAME)) {
            Ok(()) | Err(winsafe::co::ERROR::FILE_NOT_FOUND) => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

pub(crate) fn move_mouse_click(x: i32, y: i32, is_test: bool) -> anyhow::Result<()> {
    SetCursorPos(x, y)?;

    let press_event = HwKbMouse::Mouse(MOUSEINPUT {
        dwFlags: MOUSEEVENTF::LEFTDOWN,
        time: 10,
        ..Default::default()
    });

    let release_event = HwKbMouse::Mouse(MOUSEINPUT {
        dwFlags: MOUSEEVENTF::LEFTUP,
        ..Default::default()
    });

    if !is_test {
        sleep(Duration::from_millis(50));
        SendInput(&[press_event, release_event])?;
        sleep(Duration::from_secs(1));
        SendInput(&[press_event, release_event])?;
    }

    Ok(())
}

pub(crate) fn get_pos() -> anyhow::Result<()> {
    let mut prev_x = 0;
    let mut prev_y = 0;
    loop {
        let point = GetCursorPos()?;
        if prev_x != point.x || prev_y != point.y {
            println!("x: {}, y: {}", point.x, point.y);
            prev_x = point.x;
            prev_y = point.y;
        }
        sleep(Duration::from_millis(100));
    }
}
