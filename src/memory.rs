const MEMORY_SIZE: usize = 0xFFFF;

pub struct Memory {
    memory: [u8; MEMORY_SIZE],
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            memory: [0; MEMORY_SIZE],
        }
    }

    pub fn load_rom(&mut self, rom_data: Vec<u8>) {
        self.memory[0x0100..(0x0100 + rom_data.len())].copy_from_slice(&rom_data[..]);
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        self.memory[address as usize] = byte;
    }
}
