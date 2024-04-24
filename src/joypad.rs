use sdl2::keyboard::Keycode;

use crate::{
    cpu::{INTERRUPT_FLAG_ADDRESS, JOYPAD_FLAG},
    memory::Memory,
    utils::{set_flag, Address, Byte},
};

// ----- joypad controls -----
pub const JOYPAD_REGISTER_ADDRESS: Address = 0xFF00;
pub const LEFT_BUTTON: Byte = 0b1101_0010;
pub const RIGHT_BUTTON: Byte = 0b1101_0001;
pub const UP_BUTTON: Byte = 0b1101_0100;
pub const DOWN_BUTTON: Byte = 0b1101_1000;
pub const A_BUTTON: Byte = 0b1110_0001;
pub const B_BUTTON: Byte = 0b1110_0010;
pub const SELECT_BUTTON: Byte = 0b1110_0100;
pub const START_BUTTON: Byte = 0b1110_1000;

pub struct Joypad {
    last_key: Option<Keycode>,
}

impl Joypad {
    pub fn new() -> Self {
        Self { last_key: None }
    }
    /// Update button register
    pub fn update(&mut self, memory: &mut Memory) {
        match self.last_key {
            None => memory.write_byte(JOYPAD_REGISTER_ADDRESS, 0xff),
            Some(keycode) => {
                let button = match keycode {
                    Keycode::A => LEFT_BUTTON,
                    Keycode::W => UP_BUTTON,
                    Keycode::D => RIGHT_BUTTON,
                    Keycode::S => DOWN_BUTTON,
                    Keycode::J => B_BUTTON,
                    Keycode::K => A_BUTTON,
                    Keycode::U => SELECT_BUTTON,
                    Keycode::I => START_BUTTON,
                    _ => return,
                };
                memory.write_byte(JOYPAD_REGISTER_ADDRESS, !button);
            }
        }
    }

    /// Handle button press
    pub fn handle_button(&mut self, keycode: Keycode, down: bool, memory: &mut Memory) {
        match keycode {
            Keycode::A
            | Keycode::W
            | Keycode::D
            | Keycode::S
            | Keycode::J
            | Keycode::K
            | Keycode::U
            | Keycode::I => match (self.last_key, down) {
                (None, true) => {
                    self.last_key = Some(keycode);
                    let mut int_flag = memory.read_byte(INTERRUPT_FLAG_ADDRESS);
                    set_flag(&mut int_flag, JOYPAD_FLAG);
                    memory.write_byte(INTERRUPT_FLAG_ADDRESS, int_flag);
                }
                // ignore
                (None, false) | (Some(_), true) => (),
                (Some(old_key), false) => {
                    if old_key == keycode {
                        self.last_key = None;
                    }
                }
            },
            _ => (),
        }
    }
}
