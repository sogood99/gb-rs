use log::info;

use crate::utils::{bytes2word, Address, Byte, Word};

const MEMORY_SIZE: usize = 0x10000;

const DMA_ADDRESS: Address = 0xFF46;
const MBC_TYPE_ADDRESS: Address = 0x0147;

pub enum RomType {
    RomOnly,
    MBC1,
}

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

    pub fn read_byte(&self, address: Address) -> Byte {
        let address = address as usize;
        self.memory[address]
    }

    pub fn read_word(&self, address: Address) -> Word {
        let address = address as usize;
        bytes2word(self.memory[address], self.memory[address + 1])
    }

    /// Write byte to address according to MMU
    pub fn write_byte(&mut self, address: Address, byte: Byte) {
        let address = address as usize;
        self.memory[address] = byte;
    }

    pub fn get_rom_type(&self) -> RomType {
        let rom_type = self.read_byte(MBC_TYPE_ADDRESS);
        match rom_type {
            0x00 => RomType::RomOnly,
            0x01 => RomType::MBC1,
            _ => unimplemented!(),
        }
    }

    /// Wrapping add value to address
    pub fn wrapping_add(&mut self, address: Address, value: Byte) {
        assert!((address as usize) < MEMORY_SIZE);
        let mut mem_val = self.read_byte(address);
        mem_val = mem_val.wrapping_add(value);
        self.write_byte(address, mem_val);
    }
}
