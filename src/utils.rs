use std::time::Duration;

use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind};

#[derive(Copy, Clone, PartialEq)]
pub(super) struct Pos {
    pub(super) x: i32,
    pub(super) y: i32,
}

impl Pos {
    pub(super) fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl std::ops::Add<(i32, i32)> for Pos {
    type Output = Pos;

    fn add(self, rhs: (i32, i32)) -> Self::Output {
        Self {
            x: self.x + rhs.0,
            y: self.y + rhs.1,
        }
    }
}

pub(super) fn get_key() -> KeyCode {
    let _ = crossterm::terminal::enable_raw_mode();
    loop {
        if poll(Duration::from_millis(1000)).unwrap() {
            let event = read().unwrap();
            match event {
                Event::Key(ev) if ev.kind == KeyEventKind::Press => {
                    let _ = crossterm::terminal::disable_raw_mode();
                    return ev.code;
                }
                _ => (),
            }
        }
    }
}
