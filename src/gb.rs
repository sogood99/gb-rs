use std::{io, time::Duration};

use log::debug;
use sdl2::{event::Event, keyboard::Keycode};

use crate::{clock::Clock, cpu::CPU, graphics::Graphics, memory::Memory};

pub struct GameBoy {
    cpu: CPU,
    memory: Memory,
    graphics: Option<Graphics>,
    clock: Clock,
    dbg: Debugger,
}

struct Debugger {
    pause: bool,
    step: bool,
}

impl Debugger {
    fn new() -> Self {
        Self {
            pause: false,
            step: false,
        }
    }

    fn toggle_pause(&mut self) {
        self.pause = !self.pause;
    }

    fn toggle_step(&mut self) {
        self.step = true;
        self.pause = false;
    }

    fn check_pause(&mut self) -> bool {
        if self.pause {
            true
        } else if self.step {
            // step for one step, and pause
            self.pause = true;
            self.step = false;
            false
        } else {
            false
        }
    }
}

impl GameBoy {
    pub fn new(graphics_enabled: bool) -> Self {
        GameBoy {
            cpu: CPU::new_skip_boot(),
            memory: Memory::new(),
            graphics: if graphics_enabled {
                Some(Graphics::new())
            } else {
                None
            },
            clock: Clock::new(),
            dbg: Debugger::new(),
        }
    }

    pub fn load_rom(&mut self, rom_data: Vec<u8>) {
        self.memory.load_rom(rom_data);
    }

    pub fn load_boot(&mut self, boot_data: Vec<u8>) {
        self.memory.load_rom(boot_data);
    }

    pub fn run(mut self) {
        loop {
            // non gb related keydowns
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
                        Event::KeyDown {
                            keycode: Some(Keycode::P),
                            ..
                        } => self.dbg.toggle_pause(),
                        Event::KeyDown {
                            keycode: Some(Keycode::RightBracket),
                            ..
                        } => self.dbg.toggle_step(),
                        _ => {}
                    }
                }
            }

            if self.dbg.check_pause() {
                continue;
            }

            let cycles = self.cpu.execute(&mut self.memory);

            self.clock.tick(cycles, &mut self.memory);

            self.cpu.handle_interrupts(&mut self.memory);

            // serial output debug
            if self.memory.read_byte_unsafe(0xff02) != 0 {
                let c = self.memory.read_byte_unsafe(0xff01) as char;
                print!("{}", c);
                self.memory.write_byte(0xff02, 0);
            }

            // render graphics
            if let Some(ref mut graphics) = self.graphics {
                // polling user inputs
                for event in graphics.event_pump.poll_iter() {
                    match event {
                        _ => {}
                    }
                }
                graphics.render();
            }

            // run audio
        }
    }
}
