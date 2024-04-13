use crate::{cpu::CPU, graphics::Graphics, memory::Memory};

pub struct GameBoy {
    cpu: CPU,
    memory: Memory,
}

impl GameBoy {
    pub fn new() -> Self {
        GameBoy {
            cpu: CPU::new(),
            memory: Memory::new(),
        }
    }

    pub fn load_rom(&mut self, rom_data: Vec<u8>) {
        self.memory.load_rom_offset(rom_data, 0x100);
    }

    pub fn load_boot(&mut self, boot_data: Vec<u8>) {
        self.memory.load_rom(boot_data);
    }

    pub fn run(mut self) {
        // initialize
        // let graphics = Graphics::new();

        for _i in 0..10 {
            // loop {
            self.cpu.execute(&mut self.memory);

            self.cpu.handle_interrupts(&mut self.memory);

            // render graphics
            // run audio
        }
    }
}
