use log::debug;

use crate::utils::{to_word, Address, Byte, ByteOP, Word};

const MEMORY_SIZE: usize = 0x10000;

pub struct Memory {
    memory: [Byte; MEMORY_SIZE],
}

pub struct MemoryReadError {}

impl Memory {
    pub fn new() -> Self {
        Memory {
            memory: [0; MEMORY_SIZE],
        }
    }

    pub fn load_rom(&mut self, rom_data: Vec<Byte>) {
        debug!("Size of rom is 0x{:02x}", rom_data.len());

        self.memory[..rom_data.len()].copy_from_slice(&rom_data[..]);
    }

    pub fn read_byte(&self, address: Address) -> Option<Byte> {
        let address = address as usize;
        if address < self.memory.len() {
            Some(self.memory[address])
        } else {
            None
        }
    }

    pub fn read_byte_unsafe(&self, address: Address) -> Byte {
        self.memory[address as usize]
    }

    pub fn read_word(&self, address: Address) -> Option<Word> {
        let address = address as usize;
        if address + 1 < self.memory.len() {
            Some(to_word(self.memory[address], self.memory[address + 1]))
        } else {
            None
        }
    }

    pub fn read_word_unsafe(&self, address: Address) -> Word {
        let address = address as usize;
        to_word(self.memory[address], self.memory[address + 1])
    }

    pub fn write_byte(&mut self, address: Address, byte: Byte) -> Option<()> {
        let address = address as usize;
        if address < self.memory.len() {
            self.memory[address] = byte;
            Some(())
        } else {
            None
        }
    }

    pub fn write_byte_unsafe(&mut self, address: Address, byte: Byte) {
        self.memory[address as usize] = byte;
    }
}
