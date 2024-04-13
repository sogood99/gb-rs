use log::debug;
use sdl2::{event::Event, keyboard::Keycode};

use crate::{cpu::CPU, graphics::Graphics, memory::Memory};

pub struct GameBoy {
    cpu: CPU,
    memory: Memory,
    graphics: Option<Graphics>,
}

impl GameBoy {
    pub fn new(graphics_enabled: bool) -> Self {
        GameBoy {
            cpu: CPU::new(),
            memory: Memory::new(),
            graphics: if graphics_enabled {
                Some(Graphics::new())
            } else {
                None
            },
        }
    }

    pub fn load_rom(&mut self, rom_data: Vec<u8>) {
        self.memory.load_rom_offset(rom_data, 0x100);
    }

    pub fn load_boot(&mut self, boot_data: Vec<u8>) {
        self.memory.load_rom(boot_data);
    }

    pub fn run(mut self) {
        // polling user inputs
        loop {
            self.cpu.execute(&mut self.memory);

            self.cpu.handle_interrupts(&mut self.memory);

            if self.memory.read_byte_unsafe(0xff02) == 0x81 {
                let c = self.memory.read_byte_unsafe(0xff01);
                print!("{}", c);
                self.memory.write_byte(0xff02, 0);
            }

            // render graphics
            if let Some(ref mut graphics) = self.graphics {
                for event in graphics.event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Q),
                            ..
                        } => return,
                        _ => {}
                    }
                }
                graphics.render();
            }
            // run audio
        }
    }
}
