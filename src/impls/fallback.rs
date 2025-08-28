use std::{thread::sleep, time::Duration};

use enigo::{Enigo, Mouse};

pub(crate) fn move_mouse_click(x: i32, y: i32, is_test: bool) -> anyhow::Result<()> {
    let mut eg = Enigo::new(&Default::default())?;
    eg.move_mouse(x, y, enigo::Coordinate::Abs)?;

    if is_test {
        eg.button(enigo::Button::Left, enigo::Direction::Click)?;
        sleep(Duration::from_secs(1));
        eg.button(enigo::Button::Left, enigo::Direction::Click)?;
    }
    Ok(())
}

pub(crate) fn get_pos() -> anyhow::Result<()> {
    let eg = enigo::Enigo::new(&enigo::Settings::default())?;
    let mut prev_x = 0;
    let mut prev_y = 0;

    loop {
        let (x, y) = eg.location()?;
        if prev_x != x || prev_y != y {
            println!("x: {x}, y: {y}");
            prev_x = x;
            prev_y = y;
        }
        sleep(Duration::from_millis(100));
    }
}
