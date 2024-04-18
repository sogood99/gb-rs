use std::{io, time::Duration};

use log::{debug, info};
use sdl2::{event::Event, keyboard::Keycode};

use crate::{
    clock::Clock,
    cpu::{Instruction, SizedInstruction, CPU},
    graphics::Graphics,
    memory::{self, Memory},
    utils::Address,
};

pub struct GameBoy {
    cpu: CPU,
    memory: Memory,
    graphics: Option<Graphics>,
    clock: Clock,
    dbg: Debugger,
}

/// Struct to hold all debugger constructs
struct Debugger {
    pause: bool,
    step: bool,
    breakpoints: Vec<Breakpoint>,
}

#[derive(Debug, PartialEq, Eq)]
enum Breakpoint {
    Inst(Instruction),
    Addr(Address),
}

impl Debugger {
    fn new() -> Self {
        Self {
            pause: false,
            step: false,
            breakpoints: Vec::new(),
        }
    }

    fn toggle_pause(&mut self) {
        self.pause = !self.pause;
    }

    fn toggle_step(&mut self) {
        self.step = true;
        self.pause = false;
    }

    fn add_breakpoint(&mut self, breakpoint: Breakpoint) {
        self.breakpoints.push(breakpoint);
    }

    fn check_breakpoints(&self, cpu: &CPU, memory: &Memory) -> bool {
        let instruction = SizedInstruction::decode(memory, cpu.pc)
            .unwrap()
            .instruction;
        self.breakpoints.contains(&Breakpoint::Inst(instruction))
            || self.breakpoints.contains(&Breakpoint::Addr(cpu.pc))
    }

    /// Check if
    fn check_pause(&mut self, cpu: &CPU, memory: &Memory) -> bool {
        if self.pause {
            true
        } else if self.step {
            // step for one step, and pause
            self.pause = true;
            self.step = false;
            false
        } else if self.check_breakpoints(cpu, memory) {
            self.pause = true;
            info!("Breakpoint: {:#04X?}", cpu.pc);
            cpu.display_registers(false);
            true
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
        // self.dbg.add_breakpoint(Breakpoint::Addr(0x50));
        // self.dbg.add_breakpoint(Breakpoint::Inst(Instruction::EI));

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

            if self.dbg.check_pause(&self.cpu, &self.memory) {
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
