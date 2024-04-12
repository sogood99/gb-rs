use log::debug;

use crate::{
    memory::Memory,
    utils::{Address, Byte, ByteOP, SignedByte, Word},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HL,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Register16 {
    BC,
    DE,
    HL,
    SP,
    AF,
}

impl Register {
    /// Assumes the register values are 0bxxx
    pub fn get_r(code: Byte) -> Self {
        match code.mask(0b111) {
            0 => Self::B,
            1 => Self::C,
            2 => Self::D,
            3 => Self::E,
            4 => Self::H,
            5 => Self::L,
            6 => Self::HL,
            7 => Self::A,
            c @ _ => panic!("Unknown Register {} for code {}", c, code),
        }
    }

    /// Assumes the register values are 0bxxxyyy
    pub fn get_rr(code: Byte) -> (Self, Self) {
        let lr_code = (code.mask(0b111 << 3) >> 3) as Byte;
        let rr_code = code.mask(0b111) as Byte;
        (Self::get_r(lr_code), Self::get_r(rr_code))
    }
}

impl Register16 {
    /// Assumes the register values are 0bxx, output the corresponding reg/regpair
    pub fn get_rr(code: Byte, push_pop: bool) -> Self {
        match code.mask(0b11) {
            0 => Self::BC,
            1 => Self::DE,
            2 => Self::HL,
            3 if !push_pop => Self::SP,
            3 if push_pop => Self::AF,
            c @ _ => panic!("Unknown Register {} for code {}", c, code),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Condition {
    NonZero,
    Zero,
    NotCarry,
    Carry,
}

#[derive(Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum Instruction {
    LD_R_R(Register, Register),
    LD_R_N(Register, Byte),
    LD_R_HL(Register),
    LD_HL_R(Register),
    LD_HL_N(Byte),
    /// Load accumulator BC
    LD_A_BC,
    /// Load accumulator DE
    LD_A_DE,
    /// Load from accumulator BC
    LD_BC_A,
    /// Load from accumulator DE
    LD_DE_A,
    /// Load accumulator
    LD_A_NN(Address),
    /// Load from accumulator
    LD_NN_A(Address),
    /// Load accumulator (indirect 0xFF00+C)
    LDH_A_C,
    /// Load from accumulator (indirect 0xFF00+C)
    LDH_C_A,
    /// Load accumulator (direct 0xFF00+n)
    LDH_A_N(Byte),
    /// Load from accumulator (indirect 0xFF00+n)
    LDH_N_A(Byte),
    /// Load accumulator (indirect HL, decrement)
    LD_A_HL_D,
    /// Load accumulator (indirect HL, increment)
    LD_A_HL_I,
    /// Load from accumulator (indirect HL, decrement)
    LD_HL_A_D,
    /// Load from accumulator (indirect HL, increment)
    LD_HL_A_I,
    /// Load 16-bit register / register pair
    LD_RR_NN(Register16, Word),
    /// Load from stack pointer (direct)
    LD_NN_SP(Word),
    ///  Load stack pointer from HL
    LD_SP_HL,
    ///  Load HL from adjusted stack pointer
    LD_HL_SP(SignedByte),
    /// Push to stack
    PUSH(Register16),
    /// Pop from stack
    POP(Register16),
    ADD_R(Register),
    ADD_HL,
    ADD_N(Byte),
    ADC_R,
    ADC_HL(Address),
    ADC_N(Address),
    SUB_R(Register),
    SUB_HL,
    SUB_N(Byte),
    SBC_R,
    SBC_HL(Address),
    SBC_N(Address),
    CP_R(Register),
    CP_HL,
    CP_N(Byte),
    INC_R(Register),
    INC_HL,
    DEC_R(Register),
    DEC_HL,
    AND_R(Register),
    AND_HL,
    AND_N(Byte),
    OR_R(Register),
    OR_HL,
    OR_N(Byte),
    XOR_R(Register),
    XOR_HL,
    XOR_N(Byte),
    JP_NN(Address),
    JP_HL,
    JP_CC_NN(Condition, Address),
    JP_E(Byte),
    JR_CC_E(Condition, Byte),
    CALL_NN(Address),
    CALL_CC_NN(Condition, Address),
    RET,
    RET_CC(Condition),
    EI,
    NOP,
    HALT,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SizedInstruction {
    pub instruction: Instruction,
    pub size: usize,
}

/// OpCode template with its effective fields
#[derive(Debug, PartialEq, Eq)]
pub struct OpCode(Byte, Byte);

impl OpCode {
    /// Check if the give opcode `code` matches self, considering mask
    fn matches(&self, code: Byte) -> bool {
        code.mask(self.1) == self.0
    }
}

impl SizedInstruction {
    // ----- opcodes , left is pattern, right is mask -----
    const NOP: OpCode = OpCode(0, 0b11111111);
    /// LOAD for RR, RHL, HLR,
    const LD1: OpCode = OpCode(0b01000000, 0b11000000);
    /// LOAD for RN or HL N
    const LD2: OpCode = OpCode(0b00000110, 0b11000111);
    /// LOAD for A_NN or NN_A
    const LD3: OpCode = OpCode(0b11101010, 0b11101111);
    /// LOAD for LDH A_C or C_A
    const LD4: OpCode = OpCode(0b11100010, 0b11101111);
    /// LOAD for LDH A_N or N_A
    const LD5: OpCode = OpCode(0b11100000, 0b11101111);
    /// LOAD for LD BC/DE/HL+/HL- to A vice versa
    const LD6: OpCode = OpCode(0b00000010, 0b11000111);
    /// LOAD for LD RR NN
    const LD7: OpCode = OpCode(0b00000001, 0b11001111);
    /// LOAD for LD NN SP
    const LD8: OpCode = OpCode(0b00001000, 0b11111111);
    /// LOAD for LD SP_HL or HL_SP+E
    const LD9: OpCode = OpCode(0b11111000, 0b11111110);
    /// PUSH or POP
    const PUSH_POP: OpCode = OpCode(0b11000001, 0b11001011);

    pub fn decode(memory: &mut Memory, address: Address) -> Option<Self> {
        let opcode = memory.read_byte(address)?;
        let (instruction, size) = if Self::NOP.matches(opcode) {
            (Instruction::NOP, 1)
        } else if Self::LD1.matches(opcode) {
            let (lr, rr) = Register::get_rr(opcode);
            let instruction = match (lr, rr) {
                (Register::HL, Register::HL) => Instruction::HALT,
                (Register::HL, r) => Instruction::LD_HL_R(r),
                (l, Register::HL) => Instruction::LD_R_HL(l),
                (l, r) => Instruction::LD_R_R(l, r),
            };
            (instruction, 1)
        } else if Self::LD2.matches(opcode) {
            let r = Register::get_r(opcode >> 3);
            let n = memory.read_byte(address + 1)?;
            let instruction = match r {
                Register::HL => Instruction::LD_HL_N(n),
                reg => Instruction::LD_R_N(reg, n),
            };
            (instruction, 2)
        } else if Self::LD3.matches(opcode) {
            let nn = memory.read_word(address + 1)?;
            let instruction = if opcode & 1 << 4 != 0 {
                Instruction::LD_A_NN(nn)
            } else {
                Instruction::LD_NN_A(nn)
            };
            (instruction, 3)
        } else if Self::LD4.matches(opcode) {
            let instruction = if opcode & 1 << 4 != 0 {
                Instruction::LDH_A_C
            } else {
                Instruction::LDH_C_A
            };
            (instruction, 1)
        } else if Self::LD5.matches(opcode) {
            let n = memory.read_byte(address + 1)?;
            let instruction = if opcode & 1 << 4 != 0 {
                Instruction::LDH_A_N(n)
            } else {
                Instruction::LDH_N_A(n)
            };
            (instruction, 2)
        } else if Self::LD6.matches(opcode) {
            let instruction = if opcode & 1 << 3 != 0 {
                // A_x case
                match opcode.get_high_nibble() {
                    0 => Instruction::LD_A_BC,
                    1 => Instruction::LD_A_DE,
                    2 => Instruction::LD_A_HL_I,
                    3 => Instruction::LD_A_HL_D,
                    _ => panic!("Nibble cannot have more than 4 values"),
                }
            } else {
                // x_A case
                match opcode.get_high_nibble() {
                    0 => Instruction::LD_BC_A,
                    1 => Instruction::LD_DE_A,
                    2 => Instruction::LD_HL_A_I,
                    3 => Instruction::LD_HL_A_D,
                    _ => panic!("Nibble cannot have more than 4 values"),
                }
            };
            (instruction, 1)
        } else if Self::LD7.matches(opcode) {
            let rr = Register16::get_rr(opcode >> 3, false);
            let nn = memory.read_word(address + 1)?;
            let instruction = Instruction::LD_RR_NN(rr, nn);
            (instruction, 3)
        } else if Self::LD8.matches(opcode) {
            let nn = memory.read_word(address + 1)?;
            let instruction = Instruction::LD_NN_SP(nn);
            (instruction, 3)
        } else if Self::LD9.matches(opcode) {
            if opcode & 1 == 1 {
                (Instruction::LD_SP_HL, 1)
            } else {
                let e = memory.read_byte(address + 1)? as SignedByte;
                (Instruction::LD_HL_SP(e), 2)
            }
        } else if Self::PUSH_POP.matches(opcode) {
            let rr = Register16::get_rr(opcode >> 4, true);
            if opcode & (1 << 2) != 0 {
                (Instruction::PUSH(rr), 1)
            } else {
                (Instruction::POP(rr), 1)
            }
        } else {
            return None;
        };
        Some(SizedInstruction { instruction, size })
    }
}

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
        unimplemented!()
        // let opcode = memory.read_byte(self.pc);
        // self.pc += 1;
        // match opcode {
        //     // NOP instruction (0x00)
        //     Self::NOP => {
        //         // Do nothing
        //         debug!("NOP Instruction");
        //     }
        //     // ADD instructions
        //     Self::ADD_R | Self::ADD_HL | Self::ADD_N => {
        //         let (result, overflow) = match opcode {
        //             Self::ADD_R => self.a.overflowing_add(self.b),
        //             Self::ADD_HL => {
        //                 let address = self.get_hl();
        //                 let value = memory.read_byte(address);
        //                 self.a.overflowing_add(value)
        //             }
        //             Self::ADD_N => {
        //                 let value = memory.read_byte(self.pc);
        //                 self.pc += 1;
        //                 self.a.overflowing_add(value)
        //             }
        //             _ => {
        //                 panic!("Will never happen")
        //             }
        //         };
        //
        //         self.a = result;
        //
        //         // Update flags
        //         self.clear_flag(Self::ZERO_FLAG);
        //         self.clear_flag(Self::SUBTRACT_FLAG);
        //         self.clear_flag(Self::HALF_CARRY_FLAG);
        //
        //         if overflow {
        //             self.set_flag(Self::CARRY_FLAG);
        //         }
        //         if result == 0 {
        //             self.set_flag(Self::ZERO_FLAG);
        //         }
        //         if (self.a & 0x0F) + (self.b & 0x0F) > 0x0F {
        //             self.set_flag(Self::HALF_CARRY_FLAG);
        //         }
        //
        //         debug!("Add instruction");
        //         self.display_registers();
        //     }
        //     // SUB/CMP instructions
        //     Self::SUB_R | Self::SUB_HL | Self::SUB_N | Self::CMP_R | Self::CMP_HL | Self::CMP_N => {
        //         let (result, overflow) = match opcode {
        //             Self::SUB_R | Self::CMP_R => self.a.overflowing_sub(self.b),
        //             Self::SUB_HL | Self::CMP_HL => {
        //                 let address = self.get_hl();
        //                 let value = memory.read_byte(address);
        //                 self.a.overflowing_sub(value)
        //             }
        //             Self::SUB_N | Self::CMP_N => {
        //                 let value = memory.read_byte(self.pc);
        //                 self.pc += 1;
        //                 self.a.overflowing_sub(value)
        //             }
        //             _ => {
        //                 panic!("Will never happen")
        //             }
        //         };
        //
        //         if [Self::SUB_R, Self::SUB_HL, Self::SUB_N].contains(&opcode) {
        //             self.a = result;
        //         }
        //
        //         // Update flags
        //         self.clear_flag(Self::ZERO_FLAG);
        //         self.clear_flag(Self::HALF_CARRY_FLAG);
        //
        //         self.set_flag(Self::SUBTRACT_FLAG);
        //         if overflow {
        //             self.set_flag(Self::CARRY_FLAG);
        //         }
        //         if result == 0 {
        //             self.set_flag(Self::ZERO_FLAG);
        //         }
        //         if (self.a & 0x0F) < (self.b & 0x0F) {
        //             self.set_flag(Self::HALF_CARRY_FLAG);
        //         }
        //
        //         debug!("SUB/CMP instruction");
        //         self.display_registers();
        //     }
        //     Self::LD_R_R | Self::LD_R_HL | Self::LD_R_N => {
        //         let value = match opcode {
        //             Self::LD_R_R => self.c,
        //             Self::LD_R_HL => {
        //                 let addr = self.get_hl();
        //                 memory.read_byte(addr)
        //             }
        //             Self::LD_R_N => {
        //                 let val = memory.read_byte(self.pc);
        //                 self.pc += 1;
        //                 val
        //             }
        //             _ => panic!("Will never happen"),
        //         };
        //         self.b = value;
        //     }
        //     Self::XOR_R => {
        //         let result = self.a ^ self.b;
        //         self.a = result;
        //         self.clear_all_flags();
        //         if result == 0 {
        //             self.set_flag(Self::ZERO_FLAG);
        //         }
        //     }
        //     _ => {
        //         debug!("Unknown opcode 0x{:02x}", opcode);
        //         unimplemented!()
        //     }
        // }
    }

    pub fn handle_interrupts(&mut self, memory: &mut Memory) {}

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
}
