use log::debug;

use crate::memory::Memory;

pub struct CPU {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub f: u8,   // flag
    pub sp: u16, // stack pointer
    pub pc: u16, // program counter
}

impl CPU {
    // ----- flags -----
    const ZERO_FLAG: u8 = 0b10000000;
    const SUBTRACT_FLAG: u8 = 0b01000000;
    const HALF_CARRY_FLAG: u8 = 0b00100000;
    const CARRY_FLAG: u8 = 0b00010000;
    // ----- opcodes -----
    const NOP: u8 = 0x00;
    // ADD
    const ADD_R: u8 = 0x80;
    const ADD_HL: u8 = 0x86;
    const ADD_N: u8 = 0xC6;
    // SUB
    const SUB_R: u8 = 0x90;
    const SUB_HL: u8 = 0x96;
    const SUB_N: u8 = 0xD6;
    // CMP
    const CMP_R: u8 = 0xB8;
    const CMP_HL: u8 = 0xBE;
    const CMP_N: u8 = 0xFE;
    // XOR
    const XOR_R: u8 = 0xA8;

    pub fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            f: 0x00,
            sp: 0xFFFE,
            pc: 0x0100, // currently start at 0x0100,
        }
    }

    fn display_registers(&self) {
        debug!("Registers:");
        debug!(
            "A: {:02X}\tB: {:02X}\tC: {:02X}\tD: {:02X}",
            self.a, self.b, self.c, self.d
        );
        debug!(
            "E: {:02X}\tH: {:02X}\tL: {:02X}\tF: {:02X}",
            self.e, self.h, self.l, self.f
        );
        debug!("SP: {:04X}\tPC: {:04X}", self.sp, self.pc);
    }

    pub fn execute(&mut self, memory: &mut Memory) {
        let opcode = memory.read_byte(self.pc);
        self.pc += 1;
        match opcode {
            // NOP instruction (0x00)
            Self::NOP => {
                // Do nothing
                debug!("NOP Instruction");
            }
            // ADD instructions
            Self::ADD_R | Self::ADD_HL | Self::ADD_N => {
                let (result, overflow) = match opcode {
                    Self::ADD_R => self.a.overflowing_add(self.b),
                    Self::ADD_HL => {
                        let address = self.get_hl();
                        let value = memory.read_byte(address);
                        self.a.overflowing_add(value)
                    }
                    Self::ADD_N => {
                        let value = memory.read_byte(self.pc);
                        self.pc += 1;
                        self.a.overflowing_add(value)
                    }
                    _ => {
                        panic!("Will never happen")
                    }
                };

                self.a = result;

                // Update flags
                self.clear_flag(Self::ZERO_FLAG);
                self.clear_flag(Self::SUBTRACT_FLAG);
                self.clear_flag(Self::HALF_CARRY_FLAG);

                if overflow {
                    self.set_flag(Self::CARRY_FLAG);
                }
                if result == 0 {
                    self.set_flag(Self::ZERO_FLAG);
                }
                if (self.a & 0x0F) + (self.b & 0x0F) > 0x0F {
                    self.set_flag(Self::HALF_CARRY_FLAG);
                }

                debug!("Add instruction");
                self.display_registers();
            }
            // SUB/CMP instructions
            Self::SUB_R | Self::SUB_HL | Self::SUB_N | Self::CMP_R | Self::CMP_HL | Self::CMP_N => {
                let (result, overflow) = match opcode {
                    Self::SUB_R | Self::CMP_R => self.a.overflowing_sub(self.b),
                    Self::SUB_HL | Self::CMP_HL => {
                        let address = self.get_hl();
                        let value = memory.read_byte(address);
                        self.a.overflowing_sub(value)
                    }
                    Self::SUB_N | Self::CMP_N => {
                        let value = memory.read_byte(self.pc);
                        self.pc += 1;
                        self.a.overflowing_sub(value)
                    }
                    _ => {
                        panic!("Will never happen")
                    }
                };

                if [Self::SUB_R, Self::SUB_HL, Self::SUB_N].contains(&opcode) {
                    self.a = result;
                }

                // Update flags
                self.clear_flag(Self::ZERO_FLAG);
                self.clear_flag(Self::HALF_CARRY_FLAG);

                self.set_flag(Self::SUBTRACT_FLAG);
                if overflow {
                    self.set_flag(Self::CARRY_FLAG);
                }
                if result == 0 {
                    self.set_flag(Self::ZERO_FLAG);
                }
                if (self.a & 0x0F) < (self.b & 0x0F) {
                    self.set_flag(Self::HALF_CARRY_FLAG);
                }

                debug!("SUB/CMP instruction");
                self.display_registers();
            }
            _ => {
                debug!("Unknown opcode {}", opcode);
                unimplemented!()
            }
        }
    }

    pub fn handle_interrupts(&mut self, memory: &mut Memory) {
        unimplemented!()
    }

    pub fn get_word(high: u8, low: u8) -> u16 {
        ((high as u16) << 8) | (low as u16)
    }

    pub fn get_hl(&self) -> u16 {
        Self::get_word(self.h, self.l)
    }

    pub fn set_flag(&mut self, flag: u8) {
        self.f |= flag;
    }

    pub fn clear_flag(&mut self, flag: u8) {
        self.f &= !flag;
    }

    pub fn clear_all_flags(&mut self) {
        self.f = 0;
    }

    pub fn set_register(&mut self) {
        // for testing purposes
    }
}
