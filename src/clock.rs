use log::info;

use crate::{
    cpu::CPU,
    memory::Memory,
    utils::{Address, Byte},
};

pub struct Clock {
    div_counter: Byte,
    timer_counter: u32,
}

impl Clock {
    pub const DIV_ADDRESS: Address = 0xFF04;
    pub const TIMA_ADDRESS: Address = 0xFF05;
    pub const TMA_ADDRESS: Address = 0xFF06;
    pub const TAC_ADDRESS: Address = 0xFF07;
    pub const TAC_ENABLE_FLAG: Byte = 0b100;
    pub const TAC_CLOCK_SELECT: Byte = 0b11;

    pub fn new() -> Self {
        Clock {
            div_counter: 0,
            timer_counter: 0,
        }
    }

    pub fn tick(&mut self, cycles: u8, memory: &mut Memory) {
        // handle divider register
        let (new_div, overflow) = self.div_counter.overflowing_add(cycles);
        self.div_counter = new_div;
        if overflow {
            memory.wrapping_add(Self::DIV_ADDRESS, 1);
        }

        // handle tima
        let tac = memory.read_byte_unsafe(Self::TAC_ADDRESS);
        if CPU::get_memory_flag(tac, Self::TAC_ENABLE_FLAG) {
            self.timer_counter += 4 * (cycles as u32);

            let frequency = match tac & Self::TAC_CLOCK_SELECT {
                0 => 4096,
                1 => 262144,
                2 => 65536,
                3 => 16384,
                _ => panic!("Logically cannot happen"),
            };

            while self.timer_counter >= 4194304 / frequency {
                memory.wrapping_add(Self::TIMA_ADDRESS, 1);

                if memory.read_byte_unsafe(Self::TIMA_ADDRESS) == 0 {
                    // set timer interrupt and reset timer
                    let mut interrupt_flags = memory.read_byte_unsafe(CPU::INTERRUPT_FLAG_ADDRESS);
                    CPU::set_memory_flag(&mut interrupt_flags, CPU::TIMER_FLAG);
                    memory.write_byte_unsafe(CPU::INTERRUPT_FLAG_ADDRESS, interrupt_flags);

                    let tma = memory.read_byte_unsafe(Self::TMA_ADDRESS);
                    memory.write_byte_unsafe(Self::TIMA_ADDRESS, tma);
                }

                self.timer_counter -= 4194304 / frequency;
            }
        }
    }
}
