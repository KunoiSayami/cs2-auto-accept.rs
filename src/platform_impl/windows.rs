use std::{thread::sleep, time::Duration};

use winsafe::{GetCursorPos, HwKbMouse, MOUSEINPUT, SendInput, SetCursorPos, co::MOUSEEVENTF};

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
