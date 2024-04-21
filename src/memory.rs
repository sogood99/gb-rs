use log::{debug, info};

use crate::utils::{bytes2word, Address, Byte, ByteOP, Word};

const MEMORY_SIZE: usize = 0x10000;

pub struct Memory {
    memory: [Byte; MEMORY_SIZE],
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            memory: [0; MEMORY_SIZE],
        }
    }

    pub fn load_rom(&mut self, rom_data: Vec<Byte>) {
        self.load_rom_offset(rom_data, 0);
    }

    pub fn load_rom_offset(&mut self, rom_data: Vec<Byte>, offset: Address) {
        let offset = offset.into();
        info!(
            "Size of rom is {:#04X?}, loaded at offset {:#04X?}",
            rom_data.len(),
            offset
        );

        self.memory[offset..rom_data.len()].copy_from_slice(&rom_data[offset..]);
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
            Some(bytes2word(self.memory[address], self.memory[address + 1]))
        } else {
            None
        }
    }

    pub fn read_word_unsafe(&self, address: Address) -> Word {
        let address = address as usize;
        bytes2word(self.memory[address], self.memory[address + 1])
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

    pub fn wrapping_add(&mut self, address: Address, value: Byte) {
        assert!((address as usize) < MEMORY_SIZE);
        let mut mem_val = self.read_byte_unsafe(address);
        mem_val = mem_val.wrapping_add(value);
        self.write_byte_unsafe(address, mem_val);
    }

    pub fn get_flag(flag_byte: Byte, flag: Byte) -> bool {
        assert_eq!(flag.count_ones(), 1);
        (flag_byte & flag) > 0
    }

    pub fn set_flag(flag_byte: &mut Byte, flag: Byte) {
        assert_eq!(flag.count_ones(), 1);
        *flag_byte = *flag_byte | flag;
    }

    pub fn reset_flag(flag_byte: &mut Byte, flag: Byte) {
        assert_eq!(flag.count_ones(), 1);
        *flag_byte = *flag_byte & !flag;
    }

    pub fn reset_all_flags(flag_byte: &mut Byte) {
        *flag_byte = 0;
    }
}
