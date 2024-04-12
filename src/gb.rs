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
        self.memory.load_rom(rom_data);
    }

    pub fn run(mut self) {
        // initialize
        let graphics = Graphics::new();

        loop {
            self.cpu.execute(&mut self.memory);

            self.cpu.handle_interrupts(&mut self.memory);

            // render graphics
            // run audio
        }
    }
}
