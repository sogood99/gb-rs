use std::collections::{HashMap, HashSet};

use sdl2::keyboard::Keycode;

use crate::{
    cpu::{INTERRUPT_FLAG_ADDRESS, JOYPAD_FLAG},
    memory::Memory,
    utils::{get_flag, set_flag, Address, Byte},
};

// ----- joypad controls -----
pub const JOYPAD_REGISTER_ADDRESS: Address = 0xFF00;
pub const DPAD_FLAG: Byte = 0b0001_0000;
pub const BUTTONS_FLAG: Byte = 0b0010_0000;

pub const RIGHT_BUTTON: Byte = 0b1110_1110;
pub const LEFT_BUTTON: Byte = 0b1110_1101;
pub const UP_BUTTON: Byte = 0b1110_1011;
pub const DOWN_BUTTON: Byte = 0b1110_0111;
pub const A_BUTTON: Byte = 0b1101_1110;
pub const B_BUTTON: Byte = 0b1101_1101;
pub const SELECT_BUTTON: Byte = 0b1101_1011;
pub const START_BUTTON: Byte = 0b1101_0111;

pub struct Joypad {
    last_keys: HashSet<Keycode>,
    code_keys: HashMap<Byte, Keycode>,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            last_keys: HashSet::new(),
            code_keys: HashMap::from([
                (UP_BUTTON, Keycode::W),
                (DOWN_BUTTON, Keycode::S),
                (LEFT_BUTTON, Keycode::A),
                (RIGHT_BUTTON, Keycode::D),
                (B_BUTTON, Keycode::J),
                (A_BUTTON, Keycode::K),
                (SELECT_BUTTON, Keycode::U),
                (START_BUTTON, Keycode::I),
            ]),
        }
    }

    /// Update button register
    pub fn update(&mut self, memory: &mut Memory) {
        let joypad_flags = memory.read_byte(JOYPAD_REGISTER_ADDRESS);
        let new_flags = if !get_flag(joypad_flags, DPAD_FLAG) {
            let mut flag = joypad_flags | 0xF;
            for dpad in [UP_BUTTON, DOWN_BUTTON, LEFT_BUTTON, RIGHT_BUTTON] {
                if self.last_keys.contains(self.code_keys.get(&dpad).unwrap()) {
                    flag &= dpad;
                }
            }
            flag
        } else if !get_flag(joypad_flags, BUTTONS_FLAG) {
            let mut flag = joypad_flags | 0xF;
            for btn in [A_BUTTON, B_BUTTON, SELECT_BUTTON, START_BUTTON] {
                if self.last_keys.contains(self.code_keys.get(&btn).unwrap()) {
                    flag &= btn;
                }
            }
            flag
        } else {
            joypad_flags | 0xF
        };
        memory.write_byte(JOYPAD_REGISTER_ADDRESS, new_flags);
    }

    /// Handle button press
    pub fn handle_button(&mut self, keycode: Keycode, down: bool, memory: &mut Memory) {
        let joypad_flags = memory.read_byte(JOYPAD_REGISTER_ADDRESS);
        match keycode {
            Keycode::A | Keycode::W | Keycode::D | Keycode::S => {
                if down {
                    if !self.last_keys.contains(&keycode) && get_flag(joypad_flags, DPAD_FLAG) {
                        let mut int_flag = memory.read_byte(INTERRUPT_FLAG_ADDRESS);
                        set_flag(&mut int_flag, JOYPAD_FLAG);
                        memory.write_byte(INTERRUPT_FLAG_ADDRESS, int_flag);
                    }
                    self.last_keys.insert(keycode);
                } else {
                    self.last_keys.remove(&keycode);
                }
            }
            Keycode::J | Keycode::K | Keycode::U | Keycode::I => {
                if down {
                    if !self.last_keys.contains(&keycode) && get_flag(joypad_flags, BUTTONS_FLAG) {
                        let mut int_flag = memory.read_byte(INTERRUPT_FLAG_ADDRESS);
                        set_flag(&mut int_flag, JOYPAD_FLAG);
                        memory.write_byte(INTERRUPT_FLAG_ADDRESS, int_flag);
                    }
                    self.last_keys.insert(keycode);
                } else {
                    self.last_keys.remove(&keycode);
                }
            }
            _ => (),
        }
    }
}
