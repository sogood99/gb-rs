use log::{debug, info};

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
    pub fn get_rr(code: Byte, sp: bool) -> Self {
        match code.mask(0b11) {
            0 => Self::BC,
            1 => Self::DE,
            2 => Self::HL,
            3 if sp => Self::SP,
            3 if !sp => Self::AF,
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

impl Condition {
    pub fn get_cond(code: Byte) -> Self {
        match code & 0b11 {
            0 => Self::NonZero,
            1 => Self::Zero,
            2 => Self::NotCarry,
            3 => Self::Carry,
            _ => panic!("Unknown Conditonal Code {}", code & 0b11),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum Instruction {
    /// Load register (register)
    LD_R_R(Register, Register),
    /// Load register (immediate)
    LD_R_N(Register, Byte),
    /// Load register (indirect HL)
    LD_R_HL(Register),
    /// Load from register (indirect HL)
    LD_HL_R(Register),
    /// Load from immediate data (indirect HL)
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
    /// Add (register)
    ADD_R(Register),
    /// Add (indirect HL)
    ADD_HL,
    /// Add (immediate)
    ADD_N(Byte),
    /// Subtract (register)
    SUB_R(Register),
    /// Subtract (indirect HL)
    SUB_HL,
    /// Subtract (immediate)
    SUB_N(Byte),
    /// And (register)
    AND_R(Register),
    /// And (indirect HL)
    AND_HL,
    /// And (immediate)
    AND_N(Byte),
    /// Or (register)
    OR_R(Register),
    /// Or (indirect HL)
    OR_HL,
    /// Or (immediate)
    OR_N(Byte),
    /// Add with carry (register)
    ADC_R(Register),
    /// Add with carry (indirect HL)
    ADC_HL,
    /// Add with carry (immediate)
    ADC_N(Byte),
    /// Subtract with carry (register)
    SBC_R(Register),
    /// Subtract with carry (indirect HL)
    SBC_HL,
    /// Subtract with carry (immediate)
    SBC_N(Byte),
    /// XOR with carry (register)
    XOR_R(Register),
    /// XOR with carry (indirect HL)
    XOR_HL,
    /// XOR with carry (immediate)
    XOR_N(Byte),
    /// Compare with carry (register)
    CP_R(Register),
    /// Compare with carry (indirect HL)
    CP_HL,
    /// Compare with carry (immediate)
    CP_N(Byte),
    /// Increment (register)
    INC_R(Register),
    /// Increment (register 16)
    INC_RR(Register16),
    /// Increment (indirect HL)
    INC_HL,
    /// Decrement (register)
    DEC_R(Register),
    /// Decrement (register 16)
    DEC_RR(Register16),
    /// Dncrement (indirect HL)
    DEC_HL,
    /// Add (16-bit register)
    ADD_HL_RR(Register16),
    /// Add to stack pointer (relative)
    ADD_SP_E(SignedByte),

    /// --------UNFINISHED----------
    /// Rotate left circular (accumulator)
    /// Rotate right circular (accumulator)
    /// Rotate left (accumulator)
    /// Rotate right (accumulator)
    /// Rotate left circular (register)
    /// Rotate left circular (indirect HL)
    /// Rotate right circular (register)
    /// Rotate right circular (indirect HL)
    /// Rotate left (register)
    /// Rotate left (indirect HL)
    /// Rotate right (register)
    /// Rotate right (indirect HL)
    /// Shift left arithmetic (register)
    /// Shift left arithmetic (indirect HL)
    /// Shift right arithmetic (register)
    /// Shift right arithmetic (indirect HL)
    /// Swap nibbles (register)
    /// Swap nibbles (indirect HL)
    /// Shift right logical (register)
    /// Shift right logical (indirect HL)
    /// Test bit (register)
    /// Test bit (indirect HL)
    /// Reset bit (register)
    /// Reset bit (indirect HL)
    /// Set bit (register)
    /// Set bit (indirect HL)
    /// --------UNFINISHED----------

    /// Unconditional jump
    JP_NN(Address),
    /// Jump to hl
    JP_HL,
    /// Conditional Jump to nn
    JP_CC_NN(Condition, Address),
    /// Relative Jump
    JR(SignedByte),
    /// Conditional Relative Jump
    JR_CC(Condition, SignedByte),
    /// Call function
    CALL(Address),
    /// Call function conditional
    CALL_CC(Condition, Address),
    /// Return from function
    RET,
    /// Return from function conditional
    RET_CC(Condition),
    /// Return from interrupt handler
    RETI,
    /// Restart / Call function (implied)
    RST(Byte),
    /// Complement carry flag
    CCF,
    /// Set carry flag
    SCF,
    /// Decimal Adjust Accumulator
    DAA,
    /// Complement Accumulator
    CPL,
    /// Enable interrupt
    EI,
    /// Disable interrupt
    DI,
    /// No operation
    NOP,
    HALT,
    STOP,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SizedInstruction {
    pub instruction: Instruction,
    pub size: Word,
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
    /// Aritmetic OP such as ADD, SUB
    const ARITH_OP_R: OpCode = OpCode(0b10000000, 0b11001000);
    /// Aritmetic OP with carry such as ADC, SBC
    const ARITH_OP_C_R: OpCode = OpCode(0b10001000, 0b11001000);
    /// Aritmetic OP such as ADD, SUB using N
    const ARITH_OP_N: OpCode = OpCode(0b11000110, 0b11001111);
    /// Aritmetic OP with carry such as ADC, SBC using N
    const ARITH_OP_C_N: OpCode = OpCode(0b11001110, 0b11001111);
    /// Increment and decrement operators on register
    const INC_DEC_R: OpCode = OpCode(0b00000100, 0b11000110);
    /// Carry flag operations
    const CARRY: OpCode = OpCode(0b00110111, 0b11110111);
    /// Increment and decrement operators on register 16
    const INC_DEC_RR: OpCode = OpCode(0b00000011, 0b11000111);
    /// Call function
    const CALL: OpCode = OpCode(0b11000100, 0b11100110);
    /// Return
    const RET: OpCode = OpCode(0b11001001, 0b11111111);
    /// Return conditional
    const RET_CC: OpCode = OpCode(0b11000000, 0b11100111);
    /// Return interrupt
    const RETI: OpCode = OpCode(0b11011001, 0b11111111);
    /// Restart
    const RST: OpCode = OpCode(0b11000111, 0b11000111);
    /// Jump
    const JP: OpCode = OpCode(0b11000011, 0b11111111);
    /// Jump HL
    const JP_HL: OpCode = OpCode(0b11101001, 0b11111111);
    /// Jump Conditional
    const JP_CC: OpCode = OpCode(0b11000010, 0b11100111);
    /// Relative Jump
    const JR: OpCode = OpCode(0b00011000, 0b11111111);
    /// Relative Jump conditional
    const JR_CC: OpCode = OpCode(0b00100000, 0b11111111);
    /// Add 16 bit register
    const ADD_HL_RR: OpCode = OpCode(0b00001001, 0b11001111);
    /// Add SP e
    const ADD_SP_E: OpCode = OpCode(0b11101000, 0b11111111);
    /// DAA
    const DAA: OpCode = OpCode(0x27, 0b11111111);
    /// CB Prefixed Opcodes

    /// Decode the opcode at address into a SizedInstruction
    pub fn decode(memory: &mut Memory, address: Address) -> Option<Self> {
        let opcode = memory.read_byte(address)?;
        debug!("Opcode: {:#04X?}", opcode);
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
            let rr = Register16::get_rr(opcode >> 4, true);
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
            let rr = Register16::get_rr(opcode >> 4, false);
            if opcode & (1 << 2) != 0 {
                (Instruction::PUSH(rr), 1)
            } else {
                (Instruction::POP(rr), 1)
            }
        } else if Self::ARITH_OP_R.matches(opcode) {
            let r = Register::get_r(opcode);
            let instruction = match (opcode.get_high_nibble(), r) {
                (8, Register::HL) => Instruction::ADD_HL,
                (8, r) => Instruction::ADD_R(r),
                (9, Register::HL) => Instruction::SUB_HL,
                (9, r) => Instruction::SUB_R(r),
                (0xa, Register::HL) => Instruction::AND_HL,
                (0xa, r) => Instruction::AND_R(r),
                (0xb, Register::HL) => Instruction::OR_HL,
                (0xb, r) => Instruction::OR_R(r),
                _ => panic!("Unknown combination, should never happen"),
            };
            (instruction, 1)
        } else if Self::ARITH_OP_C_R.matches(opcode) {
            let r = Register::get_r(opcode);
            let instruction = match (opcode.get_high_nibble(), r) {
                (8, Register::HL) => Instruction::ADC_HL,
                (8, r) => Instruction::ADC_R(r),
                (9, Register::HL) => Instruction::SBC_HL,
                (9, r) => Instruction::SBC_R(r),
                (0xa, Register::HL) => Instruction::XOR_HL,
                (0xa, r) => Instruction::XOR_R(r),
                (0xb, Register::HL) => Instruction::CP_HL,
                (0xb, r) => Instruction::CP_R(r),
                _ => panic!("Unknown combination, should never happen"),
            };
            (instruction, 1)
        } else if Self::ARITH_OP_N.matches(opcode) {
            let n = memory.read_byte(address + 1)?;
            let instruction = match opcode.get_high_nibble() {
                0xc => Instruction::ADD_N(n),
                0xd => Instruction::SUB_N(n),
                0xe => Instruction::AND_N(n),
                0xf => Instruction::OR_N(n),
                _ => panic!("Unknown combination, should never happen"),
            };
            (instruction, 2)
        } else if Self::ARITH_OP_C_N.matches(opcode) {
            let n = memory.read_byte(address + 1)?;
            let instruction = match opcode.get_high_nibble() {
                0xc => Instruction::ADC_N(n),
                0xd => Instruction::SBC_N(n),
                0xe => Instruction::XOR_N(n),
                0xf => Instruction::CP_N(n),
                _ => panic!("Unknown combination, should never happen"),
            };
            (instruction, 2)
        } else if Self::INC_DEC_R.matches(opcode) {
            let r = Register::get_r(opcode >> 3);
            let instruction = if opcode & 1 == 0 {
                // increment
                match r {
                    Register::HL => Instruction::INC_HL,
                    r => Instruction::INC_R(r),
                }
            } else {
                match r {
                    Register::HL => Instruction::DEC_HL,
                    r => Instruction::DEC_R(r),
                }
            };
            (instruction, 1)
        } else if Self::CARRY.matches(opcode) {
            let instruction = if opcode & (1 << 3) != 0 {
                Instruction::CCF
            } else {
                Instruction::SCF
            };

            (instruction, 1)
        } else if Self::INC_DEC_RR.matches(opcode) {
            let rr = Register16::get_rr(opcode >> 4, true);
            let instruction = if opcode & (1 << 3) != 0 {
                Instruction::DEC_RR(rr)
            } else {
                Instruction::INC_RR(rr)
            };

            (instruction, 1)
        } else if Self::INC_DEC_RR.matches(opcode) {
            let rr = Register16::get_rr(opcode >> 4, true);
            let instruction = if opcode & (1 << 3) != 0 {
                Instruction::DEC_RR(rr)
            } else {
                Instruction::INC_RR(rr)
            };

            (instruction, 1)
        } else if Self::CALL.matches(opcode) {
            let nn = memory.read_word(address + 1)?;
            let instruction = if opcode & 1 != 0 {
                // ret
                Instruction::CALL(nn)
            } else {
                let cc = Condition::get_cond(opcode >> 3);
                Instruction::CALL_CC(cc, nn)
            };
            (instruction, 3)
        } else if Self::RET.matches(opcode) {
            (Instruction::RET, 1)
        } else if Self::RET_CC.matches(opcode) {
            let cc = Condition::get_cond(opcode >> 3);
            (Instruction::RET_CC(cc), 1)
        } else if Self::RETI.matches(opcode) {
            (Instruction::RETI, 1)
        } else if Self::RST.matches(opcode) {
            let n = (opcode >> 3) & 0b111;
            (Instruction::RST(n * 8), 1)
        } else if Self::JP.matches(opcode) {
            let nn = memory.read_word(address + 1)?;
            (Instruction::JP_NN(nn), 3)
        } else if Self::JP_HL.matches(opcode) {
            (Instruction::JP_HL, 1)
        } else if Self::JP_CC.matches(opcode) {
            let cc = Condition::get_cond(opcode >> 3);
            let nn = memory.read_word(address + 1)?;
            (Instruction::JP_CC_NN(cc, nn), 3)
        } else if Self::JR.matches(opcode) {
            let n = memory.read_byte(address + 1)?;
            (Instruction::JR(n as SignedByte), 2)
        } else if Self::JR_CC.matches(opcode) {
            let cc = Condition::get_cond(opcode >> 3);
            let n = memory.read_byte(address + 1)?;
            (Instruction::JR_CC(cc, n as SignedByte), 2)
        } else if Self::DAA.matches(opcode) {
            (Instruction::DAA, 1)
        } else if Self::ADD_HL_RR.matches(opcode) {
            let rr = Register16::get_rr(opcode >> 4, true);
            (Instruction::ADD_HL_RR(rr), 1)
        } else if Self::ADD_SP_E.matches(opcode) {
            let e = memory.read_byte(address + 1)? as SignedByte;
            (Instruction::ADD_SP_E(e), 2)
        } else {
            return None;
        };
        Some(SizedInstruction { instruction, size })
    }
}

pub struct CPU {
    pub a: Byte,
    pub b: Byte,
    pub c: Byte,
    pub d: Byte,
    pub e: Byte,
    pub h: Byte,
    pub l: Byte,
    pub f: Byte,  // flag
    pub sp: Word, // stack pointer
    pub pc: Word, // program counter
}

impl CPU {
    // ----- flags -----
    const ZERO_FLAG: Byte = 0b10000000;
    const SUBTRACT_FLAG: Byte = 0b01000000;
    const HALF_CARRY_FLAG: Byte = 0b00100000;
    const CARRY_FLAG: Byte = 0b00010000;

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
            pc: 0x0, // currently start at 0x00,
        }
    }

    fn get_register(&mut self, reg: Register) -> &mut Byte {
        match reg {
            Register::A => &mut self.a,
            Register::B => &mut self.b,
            Register::C => &mut self.c,
            Register::D => &mut self.d,
            Register::E => &mut self.e,
            Register::H => &mut self.h,
            Register::L => &mut self.l,
            Register::HL => panic!("Unknown register HL"),
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

    pub fn execute(&mut self, memory: &mut Memory) -> Option<()> {
        let instruction = SizedInstruction::decode(memory, self.pc)?;
        debug!("Decoded Instruction: {:#04X?}", instruction);

        match instruction.instruction {
            Instruction::NOP => (),
            Instruction::ADD_R(r) => {
                let reg_val = *self.get_register(r);
                let (result, overflow) = self.a.overflowing_add(reg_val);
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
                self.display_registers();
            }
            _ => {
                info!("Unimplemented instruction");
            }
        }

        self.pc += instruction.size;

        Some(())

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
