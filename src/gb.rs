use std::{io, time::Duration};

use log::{debug, info};
use sdl2::{
    event::{Event, EventType},
    keyboard::Keycode,
};

use crate::{
    clock::Clock,
    cpu::{Instruction, SizedInstruction, CPU},
    graphics::Graphics,
    memory::Memory,
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
            cpu: CPU::new(),
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
        self.memory.load_rom_offset(rom_data, 0x100);
    }

    pub fn load_boot(&mut self, boot_data: Vec<u8>) {
        self.memory.load_rom(boot_data);
    }

    pub fn run(mut self) {
        // self.dbg.add_breakpoint(Breakpoint::Addr(0x0c));
        // self.dbg.add_breakpoint(Breakpoint::Inst(Instruction::EI));

        // disable all events, enable only ones needed
        if let Some(ref mut graphics) = self.graphics {
            for i in 0..=65_535 {
                match EventType::try_from(i) {
                    Err(_) => (),
                    Ok(evt) => {
                        graphics.event_pump.disable_event(evt);
                    }
                }
            }
            graphics.event_pump.enable_event(EventType::Quit);
            graphics.event_pump.enable_event(EventType::KeyDown);
        }

        loop {
            if let Some(ref mut graphics) = self.graphics {
                match graphics.event_pump.poll_event() {
                    None => (),
                    Some(ev) => match ev {
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
                    },
                }
            }
            if self.dbg.check_pause(&self.cpu, &self.memory) {
                continue;
            }

            // start executing gb
            if self.cpu.halt {
                self.clock.tick(1, &mut self.memory);
            } else {
                self.cpu.execute(&mut self.memory, &mut self.clock);
            }

            self.cpu.handle_interrupts(&mut self.memory);

            self.cpu.ime_step();

            // serial output debug
            if self.memory.read_byte_unsafe(0xff02) != 0 {
                let c = self.memory.read_byte_unsafe(0xff01) as char;
                print!("{}", c);
                self.memory.write_byte(0xff02, 0);
            }

            // render graphics
            if let Some(ref mut graphics) = self.graphics {
                // non gb related keydowns

                graphics.render(&mut self.memory, self.clock.get_timestamp());
            }
            // std::thread::sleep(std::time::Duration::from_nanos(10));

            // run audio
        }
    }
}
