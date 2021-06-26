use crate::Result;
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent},
    terminal,
};
use std::time::Duration;

pub struct Keyboard {
    // Key(0-F) pressed status
    key: [bool; 16],
}

impl Keyboard {
    pub fn new() -> Result<Self> {
        // For keyboard events to work properly
        terminal::enable_raw_mode()?;

        Ok(Self {
            key: Default::default(),
        })
    }

    pub fn get(&self, k: usize) -> bool {
        self.key[k]
    }

    pub fn find_pressed_key(&self) -> Option<u8> {
        self.key
            .iter()
            .enumerate()
            .find(|(_, &v)| v)
            .map(|(k, _)| k as u8)
    }

    pub fn poll(&mut self) {
        if let Ok(true) = poll(Duration::from_millis(0)) {
            if let Ok(Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: _,
            })) = read()
            {
                match c {
                    'q' => quit(),
                    '0'..='9' | 'a'..='f' | 'A'..='F' => {
                        let i = c.to_digit(16).unwrap();
                        self.key.fill(false);
                        self.key[i as usize] = true;
                    }
                    _ => (),
                }
            }
        }
    }

    pub fn block_until_press_next() {
        loop {
            if let Ok(Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: _,
            })) = read()
            {
                match c {
                    'n' => break,
                    'q' => quit(),
                    _ => (),
                }
            }
        }
    }
}

fn quit() {
    terminal::disable_raw_mode().expect("Exit raw mode");
    std::process::exit(0)
}
