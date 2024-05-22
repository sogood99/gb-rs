use std::collections::HashSet;

use log::info;
use sdl2::{
    event::{Event, EventType},
    keyboard::Keycode,
};

use crate::{
    audio::Audio,
    clock::Clock,
    cpu::{Instruction, SizedInstruction, CPU},
    graphics::Graphics,
    joypad::Joypad,
    memory::Memory,
    utils::Address,
};

pub struct GameBoy {
    cpu: CPU,
    memory: Memory,
    graphics: Option<Graphics>,
    audio: Option<Audio>,
    clock: Clock,
    joypad: Joypad,
    dbg: Debugger,
}

/// Struct to hold all debugger constructs
struct Debugger {
    pause: bool,
    step: bool,
    breakpoints: HashSet<Breakpoint>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Breakpoint {
    Inst(Instruction),
    Addr(Address),
}

impl Debugger {
    fn new() -> Self {
        Self {
            pause: false,
            step: false,
            breakpoints: HashSet::new(),
        }
    }

    fn toggle_pause(&mut self) {
        self.pause = !self.pause;
    }

    fn toggle_step(&mut self) {
        self.step = true;
        self.pause = false;
    }

    #[allow(dead_code)]
    fn add_breakpoint(&mut self, breakpoint: Breakpoint) {
        self.breakpoints.insert(breakpoint);
    }

    fn check_breakpoints(&self, cpu: &CPU, memory: &Memory) -> bool {
        let instruction = SizedInstruction::decode(memory, cpu.pc)
            .unwrap()
            .instruction;
        self.breakpoints.contains(&Breakpoint::Inst(instruction))
            || self.breakpoints.contains(&Breakpoint::Addr(cpu.pc))
    }

    /// Check if pause, with effect
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
    pub fn new(graphics_enabled: bool, audio_enabled: bool) -> Self {
        // Initialize SDL
        let context = sdl2::init().unwrap();

        GameBoy {
            cpu: CPU::new(),
            memory: Memory::new(),
            graphics: if graphics_enabled {
                Some(Graphics::new(&context))
            } else {
                None
            },
            audio: if audio_enabled {
                Some(Audio::new(&context))
            } else {
                None
            },
            joypad: Joypad::new(),
            clock: Clock::new(),
            dbg: Debugger::new(),
        }
    }

    pub fn load_rom(&mut self, rom_data: Vec<u8>) {
        self.memory.load_cartidge(rom_data);
    }

    pub fn load_boot(&mut self, boot_data: Vec<u8>) {
        self.memory.load_boot(boot_data);
    }

    pub fn run(mut self) {
        // self.dbg.add_breakpoint(Breakpoint::Addr(0x039e));
        // self.dbg.add_breakpoint(Breakpoint::Inst(Instruction::EI));

        // timestamps and time
        let mut last_timestamp = 0;
        let mut last_time = std::time::Instant::now();
        let mut last_poll_time = std::time::Instant::now();

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
            graphics.event_pump.enable_event(EventType::KeyUp);
        }

        loop {
            // poll every 0.1s
            if let Some(ref mut graphics) = self.graphics {
                if last_poll_time.elapsed().as_millis() > 50 {
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
                            Event::KeyDown {
                                keycode: Some(k), ..
                            } => self.joypad.handle_button(k, true, &mut self.memory),
                            Event::KeyUp {
                                keycode: Some(k), ..
                            } => self.joypad.handle_button(k, false, &mut self.memory),
                            _ => {}
                        }
                    }
                    last_poll_time = std::time::Instant::now();
                }
            }
            if self.dbg.check_pause(&self.cpu, &self.memory) {
                continue;
            }

            // update joypad
            self.joypad.update(&mut self.memory);

            // start executing gb
            if self.cpu.halt {
                self.clock.tick(1, &mut self.memory);
            } else {
                self.cpu.execute(&mut self.memory, &mut self.clock);
            }

            self.cpu.handle_interrupts(&mut self.memory);

            self.cpu.ime_step();

            // serial output debug
            if self.memory.read_byte(0xff02) != 0 {
                let c = self.memory.read_byte(0xff01) as char;
                print!("{}", c);
                self.memory.write_byte(0xff02, 0);
            }

            // run audio
            if let Some(ref mut audio) = self.audio {
                if self.clock.get_timestamp() % 100 == 0 {
                    audio.handle_audio(&mut self.memory, &self.clock);
                }
            }

            // render graphics
            if let Some(ref mut graphics) = self.graphics {
                // non gb related keydowns

                graphics.render(&mut self.memory, self.clock.get_timestamp());
                if self.clock.get_timestamp() - last_timestamp > 17476 {
                    while last_time.elapsed().as_millis() < 16 {
                        graphics.timer.delay(1);
                    }
                    last_timestamp = self.clock.get_timestamp();
                    last_time = std::time::Instant::now();
                }
            }
        }
    }
}
