use log::{debug, info};

use crate::{
    clock::Clock,
    memory::Memory,
    utils::{bytes2word, get_flag, reset_flag, Address, Byte, ByteOP, SignedByte, Word, WordOP},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, PartialEq, Eq, Hash)]
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

#[derive(Debug, PartialEq, Eq, Hash)]
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
    /// Rotate left circular (accumulator)
    RLCA,
    /// Rotate right circular (accumulator)
    RRCA,
    /// Rotate left (accumulator)
    RLA,
    /// Rotate right (accumulator)
    RRA,

    /// --------CB Prefix----------
    /// Rotate left circular (register)
    RLC(Register),
    /// Rotate left circular (indirect HL)
    RLC_HL,
    /// Rotate right circular (register)
    RRC(Register),
    /// Rotate right circular (indirect HL)
    RRC_HL,
    /// Rotate left (register)
    RL(Register),
    /// Rotate left (indirect HL)
    RL_HL,
    /// Rotate right (register)
    RR(Register),
    /// Rotate right (indirect HL)
    RR_HL,
    /// Shift left arithmetic (register)
    SLA(Register),
    /// Shift left arithmetic (indirect HL)
    SLA_HL,
    /// Shift right arithmetic (register)
    SRA(Register),
    /// Shift right arithmetic (indirect HL)
    SRA_HL,
    /// Swap nibbles (register)
    SWAP(Register),
    /// Swap nibbles (indirect HL)
    SWAP_HL,
    /// Shift right logical (register)
    SRL(Register),
    /// Shift right logical (indirect HL)
    SRL_HL,
    /// Test bit (register)
    BIT(Byte, Register),
    /// Test bit (indirect HL)
    BIT_HL(Byte),
    /// Reset bit (register)
    RES(Byte, Register),
    /// Reset bit (indirect HL)
    RES_HL(Byte),
    /// Set bit (register)
    SET(Byte, Register),
    /// Set bit (indirect HL)
    SET_HL(Byte),
    /// --------CB Prefix----------

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
    const JP_CC: OpCode = OpCode(0b1100_0010, 0b1110_0111);
    /// Relative Jump
    const JR: OpCode = OpCode(0b0001_1000, 0b1111_1111);
    /// Relative Jump conditional
    const JR_CC: OpCode = OpCode(0b0010_0000, 0b1110_0111);
    /// Add 16 bit register
    const ADD_HL_RR: OpCode = OpCode(0b0000_1001, 0b1100_1111);
    /// Add SP e
    const ADD_SP_E: OpCode = OpCode(0b1110_1000, 0b1111_1111);
    /// Complement operators
    const COMP_OP: OpCode = OpCode(0b0010_1111, 0b1110_1111);
    /// DAA
    const DAA: OpCode = OpCode(0x27, 0b1111_1111);
    /// Rotate accumulator
    const ROT_ACC: OpCode = OpCode(0b0000_0111, 0b1110_0111);
    /// CB Prefixed Opcodes
    const CB: OpCode = OpCode(0xCB, 0b1111_1111);
    /// CB Prefixed Opcodes for RLC, RL, SLA, SWAP, RRC, etc
    const CB1: OpCode = OpCode(0b0000_0000, 0b1100_0000);
    /// Interrupt Opcodes
    const IR: OpCode = OpCode(0b1111_0011, 0b1111_0111);

    /// Decode the opcode at address into a SizedInstruction
    pub fn decode(memory: &Memory, address: Address) -> Option<Self> {
        let opcode = memory.read_byte(address);
        debug!("Address: {:#04X?}, Opcode: {:#04X?}", address, opcode);
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
            let n = memory.read_byte(address + 1);
            let instruction = match r {
                Register::HL => Instruction::LD_HL_N(n),
                reg => Instruction::LD_R_N(reg, n),
            };
            (instruction, 2)
        } else if Self::LD3.matches(opcode) {
            let nn = memory.read_word(address + 1);
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
            let n = memory.read_byte(address + 1);
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
            let nn = memory.read_word(address + 1);
            let instruction = Instruction::LD_RR_NN(rr, nn);
            (instruction, 3)
        } else if Self::LD8.matches(opcode) {
            let nn = memory.read_word(address + 1);
            let instruction = Instruction::LD_NN_SP(nn);
            (instruction, 3)
        } else if Self::LD9.matches(opcode) {
            if opcode & 1 == 1 {
                (Instruction::LD_SP_HL, 1)
            } else {
                let e = memory.read_byte(address + 1) as SignedByte;
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
            let n = memory.read_byte(address + 1);
            let instruction = match opcode.get_high_nibble() {
                0xc => Instruction::ADD_N(n),
                0xd => Instruction::SUB_N(n),
                0xe => Instruction::AND_N(n),
                0xf => Instruction::OR_N(n),
                _ => panic!("Unknown combination, should never happen"),
            };
            (instruction, 2)
        } else if Self::ARITH_OP_C_N.matches(opcode) {
            let n = memory.read_byte(address + 1);
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
            let nn = memory.read_word(address + 1);
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
            let nn = memory.read_word(address + 1);
            (Instruction::JP_NN(nn), 3)
        } else if Self::JP_HL.matches(opcode) {
            (Instruction::JP_HL, 1)
        } else if Self::JP_CC.matches(opcode) {
            let cc = Condition::get_cond(opcode >> 3);
            let nn = memory.read_word(address + 1);
            (Instruction::JP_CC_NN(cc, nn), 3)
        } else if Self::JR.matches(opcode) {
            let n = memory.read_byte(address + 1);
            (Instruction::JR(n as SignedByte), 2)
        } else if Self::JR_CC.matches(opcode) {
            let cc = Condition::get_cond(opcode >> 3);
            let n = memory.read_byte(address + 1);
            (Instruction::JR_CC(cc, n as SignedByte), 2)
        } else if Self::DAA.matches(opcode) {
            (Instruction::DAA, 1)
        } else if Self::ADD_HL_RR.matches(opcode) {
            let rr = Register16::get_rr(opcode >> 4, true);
            (Instruction::ADD_HL_RR(rr), 1)
        } else if Self::ADD_SP_E.matches(opcode) {
            let e = memory.read_byte(address + 1) as SignedByte;
            (Instruction::ADD_SP_E(e), 2)
        } else if Self::COMP_OP.matches(opcode) {
            let instruction = if opcode & (1 << 4) > 0 {
                Instruction::CCF
            } else {
                Instruction::CPL
            };
            (instruction, 1)
        } else if Self::ROT_ACC.matches(opcode) {
            let instruction = match opcode & (1 << 3) > 0 {
                true => match opcode & (1 << 4) > 0 {
                    true => Instruction::RRA,
                    false => Instruction::RRCA,
                },
                false => match opcode & (1 << 4) > 0 {
                    true => Instruction::RLA,
                    false => Instruction::RLCA,
                },
            };
            (instruction, 1)
        } else if Self::CB.matches(opcode) {
            let sized_instruction = Self::decode_cb(memory, address + 1);
            return match sized_instruction {
                Some(mut instruction) => {
                    instruction.size += 1;
                    Some(instruction)
                }
                None => None,
            };
        } else if Self::IR.matches(opcode) {
            let instruction = if opcode & (1 << 3) > 0 {
                Instruction::EI
            } else {
                Instruction::DI
            };
            (instruction, 1)
        } else {
            return None;
        };
        Some(SizedInstruction { instruction, size })
    }

    /// Decode CB-Prefixed instructions
    fn decode_cb(memory: &Memory, address: Address) -> Option<Self> {
        let opcode = memory.read_byte(address);
        debug!("CB-Prefixed OpCode: {:#04X?}", opcode);
        let r = Register::get_r(opcode);
        let instruction = if Self::CB1.matches(opcode) {
            if opcode & (1 << 3) > 0 {
                match opcode.get_high_nibble() {
                    0 => {
                        if r == Register::HL {
                            Instruction::RRC_HL
                        } else {
                            Instruction::RRC(r)
                        }
                    }
                    1 => {
                        if r == Register::HL {
                            Instruction::RR_HL
                        } else {
                            Instruction::RR(r)
                        }
                    }
                    2 => {
                        if r == Register::HL {
                            Instruction::SRA_HL
                        } else {
                            Instruction::SRA(r)
                        }
                    }
                    3 => {
                        if r == Register::HL {
                            Instruction::SRL_HL
                        } else {
                            Instruction::SRL(r)
                        }
                    }
                    _ => panic!("Nibble should not be > 4"),
                }
            } else {
                match opcode.get_high_nibble() {
                    0 => {
                        if r == Register::HL {
                            Instruction::RLC_HL
                        } else {
                            Instruction::RLC(r)
                        }
                    }
                    1 => {
                        if r == Register::HL {
                            Instruction::RL_HL
                        } else {
                            Instruction::RL(r)
                        }
                    }
                    2 => {
                        if r == Register::HL {
                            Instruction::SLA_HL
                        } else {
                            Instruction::SLA(r)
                        }
                    }
                    3 => {
                        if r == Register::HL {
                            Instruction::SWAP_HL
                        } else {
                            Instruction::SWAP(r)
                        }
                    }
                    _ => panic!("Nibble should not be > 4"),
                }
            }
        } else {
            let b = (opcode >> 3) & 0b111;
            let r = Register::get_r(opcode & 0b111);
            match opcode >> 6 {
                1 => {
                    // BIT x,r
                    if r == Register::HL {
                        Instruction::BIT_HL(b)
                    } else {
                        Instruction::BIT(b, r)
                    }
                }
                2 => {
                    // RES x,r
                    if r == Register::HL {
                        Instruction::RES_HL(b)
                    } else {
                        Instruction::RES(b, r)
                    }
                }
                3 => {
                    // SET x,r
                    if r == Register::HL {
                        Instruction::SET_HL(b)
                    } else {
                        Instruction::SET(b, r)
                    }
                }
                _ => panic!("Should not be contain any other cases {:#04X?}", opcode),
            }
        };
        Some(SizedInstruction {
            instruction,
            size: 1,
        })
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
    pub f: Byte,                    // flag
    pub sp: Word,                   // stack pointer
    pub pc: Word,                   // program counter
    pub ime: (Option<usize>, bool), // Interrupt Master Enable Flag, left is countdown (if exists), right is the flag
    pub halt: bool,                 // Halt flag
}

impl CPU {
    // ----- flags -----
    pub const ZERO_FLAG: Byte = 0b10000000;
    pub const SUBTRACT_FLAG: Byte = 0b01000000;
    pub const HALF_CARRY_FLAG: Byte = 0b00100000;
    pub const CARRY_FLAG: Byte = 0b00010000;

    // ----- memory flag -----
    pub const INTERRUPT_FLAG_ADDRESS: Address = 0xFF0F;
    pub const INTERRUPT_ENABLE_ADDRESS: Address = 0xFFFF;
    pub const VBLANK_FLAG: Byte = 0b1;
    pub const LCD_FLAG: Byte = 0b10;
    pub const TIMER_FLAG: Byte = 0b100;
    pub const SERIAL_FLAG: Byte = 0b1000;
    pub const JOYPAD_FLAG: Byte = 0b10000;

    pub fn new() -> Self {
        Self {
            a: 0x00,
            f: 0x00,
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            h: 0x00,
            l: 0x00,
            sp: 0x00,
            pc: 0x00, // currently start at 0x00,
            ime: (None, false),
            halt: false,
        }
    }

    pub fn new_skip_boot() -> Self {
        // skip the boot step, and set the register results
        Self {
            a: 0x01,
            f: 0xb0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xd8,
            h: 0x01,
            l: 0x4d,
            sp: 0xfffe,
            pc: 0x100, // currently start at 0x100,
            ime: (None, false),
            halt: false,
        }
    }

    /// Execute the instruction, and return the clock cycles used
    pub fn execute(&mut self, memory: &mut Memory, clock: &mut Clock) {
        let instruction = match SizedInstruction::decode(memory, self.pc) {
            Some(ins) => ins,
            None => panic!("Could not decode {:#04X?}", memory.read_byte(self.pc)),
        };

        debug!(
            "Decoded Instruction: {:?} {:#04X?}",
            instruction, instruction
        );
        match instruction.instruction {
            Instruction::NOP => {
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::ADD_R(r) => {
                let reg_val = self.get_register(r);
                let (result, overflow) = self.a.overflowing_add(reg_val);

                // Update flags
                self.zero_flag(result);
                self.half_carry_flag_add(self.a, reg_val);
                self.reset_flag(Self::SUBTRACT_FLAG);

                self.reset_flag(Self::CARRY_FLAG);
                if overflow {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.a = result;
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::ADD_N(n) => {
                let (result, overflow) = self.a.overflowing_add(n);
                self.zero_flag(result);
                self.half_carry_flag_add(self.a, n);
                self.reset_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::CARRY_FLAG);
                if overflow {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.a = result;
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::ADD_HL => {
                let value = memory.read_byte(self.get_hl());
                let (result, overflow) = self.a.overflowing_add(value);
                self.zero_flag(result);
                self.half_carry_flag_add(self.a, value);
                self.reset_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::CARRY_FLAG);
                if overflow {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.a = result;
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::SUB_R(r) => {
                let reg_val = self.get_register(r);
                let (result, overflow) = self.a.overflowing_sub(reg_val);

                self.zero_flag(result);
                self.half_carry_flag_sub(self.a, reg_val);
                self.set_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::CARRY_FLAG);
                if overflow {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.a = result;
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::SUB_N(n) => {
                let (result, overflow) = self.a.overflowing_sub(n);

                self.zero_flag(result);
                self.half_carry_flag_sub(self.a, n);
                self.set_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::CARRY_FLAG);
                if overflow {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.a = result;
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::SUB_HL => {
                let val = memory.read_byte(self.get_hl());
                let (result, overflow) = self.a.overflowing_sub(val);

                self.zero_flag(result);
                self.half_carry_flag_sub(self.a, val);
                self.set_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::CARRY_FLAG);
                if overflow {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.a = result;
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::AND_R(r) => {
                let result = self.a & self.get_register(r);
                self.a = result;
                self.zero_flag(result);
                self.set_flag(Self::HALF_CARRY_FLAG);
                self.reset_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::CARRY_FLAG);
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::AND_N(n) => {
                let result = self.a & n;
                self.a = result;
                self.zero_flag(result);
                self.set_flag(Self::HALF_CARRY_FLAG);
                self.reset_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::CARRY_FLAG);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::AND_HL => {
                let result = self.a & memory.read_byte(self.get_hl());
                self.a = result;
                self.zero_flag(result);
                self.set_flag(Self::HALF_CARRY_FLAG);
                self.reset_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::CARRY_FLAG);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::OR_R(r) => {
                let result = self.a | self.get_register(r);
                self.reset_all_flags();
                self.zero_flag(result);
                self.a = result;
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::OR_HL => {
                let value = memory.read_byte(self.get_hl());
                let result = self.a | value;
                self.reset_all_flags();
                self.zero_flag(result);
                self.a = result;
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::OR_N(n) => {
                let result = self.a | n;
                self.reset_all_flags();
                self.zero_flag(result);
                self.a = result;
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::XOR_R(r) => {
                let result = self.a ^ self.get_register(r);

                self.reset_all_flags();
                self.zero_flag(result);
                self.a = result;
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::XOR_HL => {
                let val = memory.read_byte(self.get_hl());
                let result = self.a ^ val;
                self.reset_all_flags();
                self.zero_flag(result);
                self.a = result;
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::XOR_N(n) => {
                let result = self.a ^ n;

                self.reset_all_flags();
                self.zero_flag(result);
                self.a = result;
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::CP_R(r) => {
                let reg_val = self.get_register(r);
                let (result, overflow) = self.a.overflowing_sub(reg_val);

                self.zero_flag(result);
                self.half_carry_flag_sub(self.a, reg_val);
                self.set_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::CARRY_FLAG);
                if overflow {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::CP_HL => {
                let address = self.get_hl();
                let val = memory.read_byte(address);
                let (result, overflow) = self.a.overflowing_sub(val);

                self.zero_flag(result);
                self.half_carry_flag_sub(self.a, val);
                self.set_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::CARRY_FLAG);
                if overflow {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::CP_N(n) => {
                let (result, overflow) = self.a.overflowing_sub(n);

                self.zero_flag(result);
                self.half_carry_flag_sub(self.a, n);
                self.set_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::CARRY_FLAG);
                if overflow {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::ADC_R(r) => {
                let reg_val = self.get_register(r);
                let cf = self.get_flag(Self::CARRY_FLAG) as Byte;
                let (res1, ovf1) = self.a.overflowing_add(reg_val);
                let (res2, ovf2) = res1.overflowing_add(cf);
                let overflow = ovf1 || ovf2;
                self.zero_flag(res2);
                self.half_carry_flag_adc(self.a, reg_val, cf);
                self.reset_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::CARRY_FLAG);
                if overflow {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.a = res2;
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::ADC_N(n) => {
                let cf = self.get_flag(Self::CARRY_FLAG) as Byte;
                let (res1, ovf1) = self.a.overflowing_add(n);
                let (res2, ovf2) = res1.overflowing_add(cf);
                let overflow = ovf1 || ovf2;
                self.zero_flag(res2);
                self.half_carry_flag_adc(self.a, n, cf);
                self.reset_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::CARRY_FLAG);
                if overflow {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.a = res2;
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::ADC_HL => {
                let val = memory.read_byte(self.get_hl());
                let cf = self.get_flag(Self::CARRY_FLAG) as Byte;
                let (res1, ovf1) = self.a.overflowing_add(val);
                let (res2, ovf2) = res1.overflowing_add(cf);
                let overflow = ovf1 || ovf2;
                self.zero_flag(res2);
                self.half_carry_flag_adc(self.a, val, cf);
                self.reset_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::CARRY_FLAG);
                if overflow {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.a = res2;
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::SBC_R(r) => {
                let reg_val = self.get_register(r);
                let cf = self.get_flag(Self::CARRY_FLAG) as Byte;
                let (res1, ovf1) = self.a.overflowing_sub(reg_val);
                let (res2, ovf2) = res1.overflowing_sub(cf);
                let overflow = ovf1 || ovf2;
                self.zero_flag(res2);
                self.half_carry_flag_sbc(self.a, reg_val, cf);
                self.set_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::CARRY_FLAG);
                if overflow {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.a = res2;
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::SBC_N(n) => {
                let cf = self.get_flag(Self::CARRY_FLAG) as Byte;
                let (res1, ovf1) = self.a.overflowing_sub(n);
                let (res2, ovf2) = res1.overflowing_sub(cf);
                let overflow = ovf1 || ovf2;
                self.zero_flag(res2);
                self.half_carry_flag_sbc(self.a, n, cf);
                self.set_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::CARRY_FLAG);
                if overflow {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.a = res2;
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::SBC_HL => {
                let val = memory.read_byte(self.get_hl());
                let cf = self.get_flag(Self::CARRY_FLAG) as Byte;
                let (res1, ovf1) = self.a.overflowing_sub(val);
                let (res2, ovf2) = res1.overflowing_sub(cf);
                let overflow = ovf1 || ovf2;
                self.zero_flag(res2);
                self.half_carry_flag_sbc(self.a, val, cf);
                self.set_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::CARRY_FLAG);
                if overflow {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.a = res2;
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::LD_R_R(r1, r2) => {
                let data = self.get_register(r2);
                self.set_register(r1, data);
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::LD_R_N(r, n) => {
                self.set_register(r, n);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::LD_R_HL(r) => {
                let data = memory.read_byte(self.get_hl());
                self.set_register(r, data);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::LD_RR_NN(rr, nn) => {
                self.set_register16(rr, nn);
                self.pc += instruction.size;
                clock.tick(3, memory);
            }
            Instruction::LD_A_HL_I => {
                self.a = memory.read_byte(self.get_hl());
                self.set_hl(self.get_hl() + 1);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::LD_A_HL_D => {
                self.a = memory.read_byte(self.get_hl());
                self.set_hl(self.get_hl() - 1);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::LDH_A_C => {
                let address = bytes2word(self.c, 0xFF);
                let data = memory.read_byte(address);
                self.a = data;
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::LDH_C_A => {
                let address = bytes2word(self.c, 0xFF);
                memory.write_byte(address, self.a);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::LD_HL_R(r) => {
                let address = self.get_hl();
                let data = self.get_register(r);
                memory.write_byte(address, data);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::LD_HL_SP(e) => {
                let e_i16: i16 = e.into();
                let e_u16 = e_i16 as Word; // first to byte (-1 => 255)
                let result = self.sp.wrapping_add_signed(e.into());
                self.reset_all_flags();
                if (self.sp & 0xF) + (e_u16 & 0xF) > 0xF {
                    self.set_flag(Self::HALF_CARRY_FLAG);
                }
                if (self.sp & 0xFF) + (e_u16 & 0xFF) > 0xFF {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.set_hl(result);
                self.pc += instruction.size;
                clock.tick(3, memory);
            }
            Instruction::LD_HL_A_D => {
                memory.write_byte(self.get_hl(), self.a);
                self.set_hl(self.get_hl() - 1);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::LD_HL_A_I => {
                memory.write_byte(self.get_hl(), self.a);
                self.set_hl(self.get_hl() + 1);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::LD_A_BC => {
                self.pc += instruction.size;
                let address = self.get_register16(Register16::BC);
                self.a = memory.read_byte(address);
                clock.tick(2, memory);
            }
            Instruction::LD_A_DE => {
                self.pc += instruction.size;
                let address = self.get_register16(Register16::DE);
                self.a = memory.read_byte(address);
                clock.tick(2, memory);
            }
            Instruction::LD_BC_A => {
                let address = self.get_register16(Register16::BC);
                memory.write_byte(address, self.a);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::LD_DE_A => {
                let address = self.get_register16(Register16::DE);
                memory.write_byte(address, self.a);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::LD_A_NN(nn) => {
                self.pc += instruction.size;
                clock.tick(2, memory);
                self.a = memory.read_byte(nn);
                clock.tick(2, memory);
            }
            Instruction::LD_NN_A(nn) => {
                clock.tick(2, memory);
                memory.write_byte(nn, self.a);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::LDH_N_A(n) => {
                self.pc += 2;
                let address = bytes2word(n, 0xFF);
                clock.tick(1, memory);
                memory.write_byte(address, self.a);
                clock.tick(2, memory);
            }
            Instruction::LDH_A_N(n) => {
                self.pc += 2;
                let address = bytes2word(n, 0xFF);
                clock.tick(1, memory);
                let data = memory.read_byte(address);
                self.a = data;
                clock.tick(2, memory);
            }
            Instruction::LD_HL_N(n) => {
                clock.tick(1, memory);
                memory.write_byte(self.get_hl(), n);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::LD_NN_SP(nn) => {
                self.pc += 3;
                memory.write_byte(nn, self.sp.get_low());
                let nn = nn + 1;
                memory.write_byte(nn, self.sp.get_high());
                clock.tick(5, memory);
            }
            Instruction::LD_SP_HL => {
                self.sp = self.get_hl();
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::INC_R(r) => {
                let reg_val = self.get_register(r);
                let (result, _overflow) = reg_val.overflowing_add(1);

                self.zero_flag(result);
                self.half_carry_flag_add(reg_val, 1);
                self.reset_flag(Self::SUBTRACT_FLAG);

                self.set_register(r, result);
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::INC_HL => {
                let val = memory.read_byte(self.get_hl());
                let (result, _overflow) = val.overflowing_add(1);

                self.zero_flag(result);
                self.half_carry_flag_add(val, 1);
                self.reset_flag(Self::SUBTRACT_FLAG);

                clock.tick(1, memory);
                memory.write_byte(self.get_hl(), result);
                clock.tick(2, memory);
                self.pc += instruction.size;
            }
            Instruction::DEC_R(r) => {
                let reg_val = self.get_register(r);
                let (result, _overflow) = reg_val.overflowing_sub(1);

                self.zero_flag(result);
                self.half_carry_flag_sub(reg_val, 1);
                self.set_flag(Self::SUBTRACT_FLAG);

                self.set_register(r, result);
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::DEC_HL => {
                let address = self.get_hl();
                let val = memory.read_byte(address);
                let (result, _overflow) = val.overflowing_sub(1);

                self.zero_flag(result);
                self.half_carry_flag_sub(val, 1);
                self.set_flag(Self::SUBTRACT_FLAG);
                clock.tick(1, memory);
                memory.write_byte(address, result);
                clock.tick(2, memory);
                self.pc += instruction.size;
            }
            Instruction::INC_RR(rr) => {
                let reg_val = self.get_register16(rr);
                let (result, _overflow) = reg_val.overflowing_add(1);
                self.set_register16(rr, result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::DEC_RR(rr) => {
                let reg_val = self.get_register16(rr);
                let (result, _overflow) = reg_val.overflowing_sub(1);
                self.set_register16(rr, result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::ADD_HL_RR(rr) => {
                let reg_val = self.get_register16(rr);
                let (result, overflow) = self.get_hl().overflowing_add(reg_val);

                self.half_carry_flag_add16(self.get_hl(), reg_val);
                self.reset_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::CARRY_FLAG);
                if overflow {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.set_hl(result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::SET(b, r) => {
                let result = self.get_register(r) | (1 << b);
                self.set_register(r, result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::SET_HL(b) => {
                clock.tick(1, memory);
                let result = memory.read_byte(self.get_hl()) | (1 << b);
                clock.tick(1, memory);
                memory.write_byte(self.get_hl(), result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::RES(b, r) => {
                let mask = !(1 << b);
                let result = self.get_register(r) & mask;
                self.set_register(r, result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::RES_HL(b) => {
                clock.tick(1, memory);
                let mask = !(1 << b);
                let result = memory.read_byte(self.get_hl()) & mask;
                clock.tick(1, memory);
                memory.write_byte(self.get_hl(), result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::BIT(b, r) => {
                let result = (self.get_register(r) & (1 << b)) >> b;
                self.reset_flag(Self::SUBTRACT_FLAG);
                self.set_flag(Self::HALF_CARRY_FLAG);
                self.zero_flag(result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::BIT_HL(b) => {
                clock.tick(1, memory);
                let result = (memory.read_byte(self.get_hl()) & (1 << b)) >> b;
                self.reset_flag(Self::SUBTRACT_FLAG);
                self.set_flag(Self::HALF_CARRY_FLAG);
                self.zero_flag(result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::CPL => {
                self.a = !self.a;
                self.set_flag(Self::SUBTRACT_FLAG);
                self.set_flag(Self::HALF_CARRY_FLAG);
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::SCF => {
                self.set_flag(Self::CARRY_FLAG);
                self.reset_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::HALF_CARRY_FLAG);
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::CCF => {
                self.reset_flag(Self::SUBTRACT_FLAG);
                self.reset_flag(Self::HALF_CARRY_FLAG);
                if self.get_flag(Self::CARRY_FLAG) {
                    self.reset_flag(Self::CARRY_FLAG);
                } else {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::DAA => {
                // turn a into decimal form, follows the official implementation
                let half_carry = self.get_flag(Self::HALF_CARRY_FLAG);
                let carry = self.get_flag(Self::CARRY_FLAG);
                let sub_flag = self.get_flag(Self::SUBTRACT_FLAG);

                if !sub_flag {
                    if carry || self.a > 0x99 {
                        self.a = self.a.wrapping_add(0x60);
                        self.set_flag(Self::CARRY_FLAG);
                    }
                    if half_carry || (self.a & 0xF) > 0x9 {
                        self.a = self.a.wrapping_add(0x6);
                    }
                } else {
                    if carry {
                        self.a = self.a.wrapping_sub(0x60);
                    }
                    if half_carry {
                        self.a = self.a.wrapping_sub(0x6);
                    }
                }
                self.reset_flag(Self::HALF_CARRY_FLAG);
                self.zero_flag(self.a);
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::JP_NN(nn) => {
                self.pc = nn;
                clock.tick(4, memory);
            }
            Instruction::JP_CC_NN(cc, nn) => {
                self.pc += 3;
                if self.get_condition(cc) {
                    self.pc = nn;
                    clock.tick(4, memory);
                } else {
                    clock.tick(3, memory);
                }
            }
            Instruction::JP_HL => {
                self.pc = self.get_hl();
                clock.tick(1, memory);
            }
            Instruction::JR(e) => {
                self.pc += 2;
                self.pc = self.pc.wrapping_add_signed(e.into());
                clock.tick(3, memory);
            }
            Instruction::JR_CC(cc, e) => {
                self.pc += 2;
                if self.get_condition(cc) {
                    self.pc = self.pc.wrapping_add_signed(e.into());
                    clock.tick(3, memory);
                } else {
                    clock.tick(2, memory);
                }
            }
            Instruction::ADD_SP_E(e) => {
                let e_i16: i16 = e.into();
                let e_u16 = e_i16 as Word; // first to byte (-1 => 255)
                let result = self.sp.wrapping_add_signed(e.into());
                self.reset_all_flags();
                if (self.sp & 0xF) + (e_u16 & 0xF) > 0xF {
                    self.set_flag(Self::HALF_CARRY_FLAG);
                }
                if (self.sp & 0xFF) + (e_u16 & 0xFF) > 0xFF {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.sp = result;
                self.pc += instruction.size;
                clock.tick(4, memory);
            }
            Instruction::PUSH(rr) => {
                self.pc += 1;
                self.sp -= 1;
                let data = self.get_register16(rr);
                memory.write_byte(self.sp, data.get_high());
                self.sp -= 1;
                memory.write_byte(self.sp, data.get_low());
                clock.tick(4, memory);
            }
            Instruction::POP(rr) => {
                self.pc += 1;
                let lsb = memory.read_byte(self.sp);
                self.sp += 1;
                let msb = memory.read_byte(self.sp);
                self.sp += 1;
                self.set_register16(rr, bytes2word(lsb, msb));
                clock.tick(3, memory);
            }
            Instruction::CALL(nn) => {
                self.pc += 3;
                self.push_pc_stack(memory);
                self.pc = nn;
                clock.tick(6, memory);
            }
            Instruction::CALL_CC(cc, nn) => {
                self.pc += 3;
                if self.get_condition(cc) {
                    self.push_pc_stack(memory);
                    self.pc = nn;
                    clock.tick(6, memory);
                } else {
                    clock.tick(3, memory);
                }
            }
            Instruction::RET => {
                self.pc += 1;
                self.pop_pc_stack(memory);
                clock.tick(4, memory);
            }
            Instruction::RET_CC(cc) => {
                self.pc += 1;
                if self.get_condition(cc) {
                    self.pop_pc_stack(memory);
                    clock.tick(5, memory);
                } else {
                    clock.tick(2, memory);
                }
            }
            Instruction::RETI => {
                self.pc += 1;
                self.pop_pc_stack(memory);
                self.ime_enable_no_delay();
                clock.tick(4, memory);
            }
            Instruction::RL(r) => {
                let reg_val = self.get_register(r);
                let old_carry = self.get_flag(Self::CARRY_FLAG) as Byte;
                let result = (reg_val << 1) | old_carry;
                self.reset_all_flags();
                self.zero_flag(result);
                if reg_val & (1 << 7) != 0 {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.set_register(r, result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::RL_HL => {
                clock.tick(1, memory);
                let val = memory.read_byte(self.get_hl());
                let old_carry = self.get_flag(Self::CARRY_FLAG) as Byte;
                let result = (val << 1) | old_carry;
                self.reset_all_flags();
                self.zero_flag(result);
                if val & (1 << 7) != 0 {
                    self.set_flag(Self::CARRY_FLAG);
                }
                clock.tick(1, memory);
                memory.write_byte(self.get_hl(), result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::RLC(r) => {
                let reg_val = self.get_register(r);
                let r7 = reg_val >> 7;
                let result = (reg_val << 1) | r7;
                self.reset_all_flags();
                self.zero_flag(result);
                if r7 != 0 {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.set_register(r, result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::RLC_HL => {
                clock.tick(1, memory);
                let val = memory.read_byte(self.get_hl());
                let r7 = val >> 7;
                let result = (val << 1) | r7;
                self.reset_all_flags();
                self.zero_flag(result);
                if r7 != 0 {
                    self.set_flag(Self::CARRY_FLAG);
                }
                clock.tick(1, memory);
                memory.write_byte(self.get_hl(), result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::RLA => {
                let r = Register::A;
                let reg_val = self.get_register(r);
                let old_carry = self.get_flag(Self::CARRY_FLAG) as Byte;
                let result = (reg_val << 1) | old_carry;
                self.reset_all_flags();
                if reg_val & (1 << 7) != 0 {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.set_register(r, result);
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::RLCA => {
                let r = Register::A;
                let reg_val = self.get_register(r);
                let r7 = reg_val >> 7;
                let result = (reg_val << 1) | r7;
                self.reset_all_flags();
                if r7 != 0 {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.set_register(r, result);
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::RR(r) => {
                let reg_val = self.get_register(r);
                let old_carry = self.get_flag(Self::CARRY_FLAG) as Byte;
                let result = (reg_val >> 1) | (old_carry << 7);
                self.reset_all_flags();
                self.zero_flag(result);
                if reg_val & 1 != 0 {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.set_register(r, result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::RR_HL => {
                clock.tick(1, memory);
                let val = memory.read_byte(self.get_hl());
                let old_carry = self.get_flag(Self::CARRY_FLAG) as Byte;
                let result = (val >> 1) | (old_carry << 7);
                self.reset_all_flags();
                self.zero_flag(result);
                if val & 1 != 0 {
                    self.set_flag(Self::CARRY_FLAG);
                }
                clock.tick(1, memory);
                memory.write_byte(self.get_hl(), result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::RRC(r) => {
                let reg_val = self.get_register(r);
                let r0 = reg_val & 1;
                let result = (reg_val >> 1) | (r0 << 7);
                self.reset_all_flags();
                self.zero_flag(result);
                if r0 != 0 {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.set_register(r, result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::RRC_HL => {
                clock.tick(1, memory);
                let val = memory.read_byte(self.get_hl());
                let r0 = val & 1;
                let result = (val >> 1) | (r0 << 7);
                self.reset_all_flags();
                self.zero_flag(result);
                if r0 != 0 {
                    self.set_flag(Self::CARRY_FLAG);
                }
                clock.tick(1, memory);
                memory.write_byte(self.get_hl(), result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::RRA => {
                let r = Register::A;
                let reg_val = self.get_register(r);
                let old_carry = self.get_flag(Self::CARRY_FLAG) as Byte;
                let result = (reg_val >> 1) | (old_carry << 7);
                self.reset_all_flags();
                if reg_val & 1 != 0 {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.set_register(r, result);
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::RRCA => {
                let r = Register::A;
                let reg_val = self.get_register(r);
                let r0 = reg_val & 1;
                let result = (reg_val >> 1) | (r0 << 7);
                self.reset_all_flags();
                if r0 != 0 {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.set_register(r, result);
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::SLA(r) => {
                let reg_val = self.get_register(r);
                let r7 = reg_val >> 7;
                let result = reg_val << 1;
                self.reset_all_flags();
                self.zero_flag(result);
                if r7 != 0 {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.set_register(r, result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::SLA_HL => {
                clock.tick(1, memory);
                let val = memory.read_byte(self.get_hl());
                let r7 = val >> 7;
                let result = val << 1;
                self.reset_all_flags();
                self.zero_flag(result);
                if r7 != 0 {
                    self.set_flag(Self::CARRY_FLAG);
                }
                clock.tick(1, memory);
                memory.write_byte(self.get_hl(), result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::SRA(r) => {
                let reg_val = self.get_register(r);
                let r7 = reg_val >> 7;
                let r0 = reg_val & 1;
                let result = (reg_val >> 1) | (r7 << 7);
                self.reset_all_flags();
                self.zero_flag(result);
                if r0 != 0 {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.set_register(r, result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::SRA_HL => {
                clock.tick(1, memory);
                let val = memory.read_byte(self.get_hl());
                let r7 = val >> 7;
                let r0 = val & 1;
                let result = (val >> 1) | (r7 << 7);
                self.reset_all_flags();
                self.zero_flag(result);
                if r0 != 0 {
                    self.set_flag(Self::CARRY_FLAG);
                }
                clock.tick(1, memory);
                memory.write_byte(self.get_hl(), result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::SRL(r) => {
                let reg_val = self.get_register(r);
                let result = reg_val >> 1;
                self.reset_all_flags();
                self.zero_flag(result);
                if reg_val & 1 != 0 {
                    self.set_flag(Self::CARRY_FLAG);
                }
                self.set_register(r, result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::SRL_HL => {
                clock.tick(1, memory);
                let val = memory.read_byte(self.get_hl());
                let result = val >> 1;
                self.reset_all_flags();
                self.zero_flag(result);
                if val & 1 != 0 {
                    self.set_flag(Self::CARRY_FLAG);
                }
                clock.tick(1, memory);
                memory.write_byte(self.get_hl(), result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::SWAP(r) => {
                let reg_val = self.get_register(r);
                let result = (reg_val >> 4) | ((reg_val & 0xf) << 4);
                self.reset_all_flags();
                self.zero_flag(result);
                self.set_register(r, result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::SWAP_HL => {
                clock.tick(1, memory);
                let val = memory.read_byte(self.get_hl());
                let result = (val >> 4) | ((val & 0xf) << 4);
                self.reset_all_flags();
                self.zero_flag(result);
                clock.tick(1, memory);
                memory.write_byte(self.get_hl(), result);
                self.pc += instruction.size;
                clock.tick(2, memory);
            }
            Instruction::RST(n) => {
                self.pc += 1;
                self.push_pc_stack(memory);
                self.pc = bytes2word(n, 0x00);
                clock.tick(4, memory);
            }
            Instruction::EI => {
                self.ime_enable();
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::DI => {
                self.ime_disable();
                self.pc += instruction.size;
                clock.tick(1, memory);
            }
            Instruction::HALT => {
                // halt bug
                self.halt = true;
                self.pc += 1;
                clock.tick(1, memory);
            }
            _ => {
                panic!(
                    "Could not execute {:#04X?} with opcode {:#04X?} at address {:#04X?}",
                    instruction,
                    memory.read_byte(self.pc),
                    self.pc
                );
            }
        };

        self.display_registers(true);
    }

    pub fn handle_interrupts(&mut self, memory: &mut Memory) {
        let interrupt_enable = memory.read_byte(Self::INTERRUPT_ENABLE_ADDRESS);
        let interrupt_flag = memory.read_byte(Self::INTERRUPT_FLAG_ADDRESS);
        let mut flag_bytes = interrupt_enable & interrupt_flag;

        // handle halt
        if flag_bytes != 0 || self.get_ime() {
            self.halt = false;
        }

        if !self.get_ime() {
            return;
        }
        if flag_bytes != 0 {
            self.ime_disable();
            self.push_pc_stack(memory);
            if get_flag(flag_bytes, Self::VBLANK_FLAG) {
                debug!("VBLANK Interrupt");
                reset_flag(&mut flag_bytes, Self::VBLANK_FLAG);
                self.pc = 0x40;
            } else if get_flag(flag_bytes, Self::LCD_FLAG) {
                debug!("LCD Interrupt");
                reset_flag(&mut flag_bytes, Self::LCD_FLAG);
                self.pc = 0x48;
            } else if get_flag(flag_bytes, Self::TIMER_FLAG) {
                debug!("TIMER Interrupt");
                reset_flag(&mut flag_bytes, Self::TIMER_FLAG);
                self.pc = 0x50;
            } else if get_flag(flag_bytes, Self::SERIAL_FLAG) {
                debug!("SERIAL Interrupt");
                reset_flag(&mut flag_bytes, Self::SERIAL_FLAG);
                self.pc = 0x58;
            } else if get_flag(flag_bytes, Self::JOYPAD_FLAG) {
                debug!("JOYPAD Interrupt");
                reset_flag(&mut flag_bytes, Self::JOYPAD_FLAG);
                self.pc = 0x60;
            }
        }
        memory.write_byte(Self::INTERRUPT_FLAG_ADDRESS, flag_bytes);
    }

    pub fn get_hl(&self) -> Word {
        self.get_register16(Register16::HL)
    }

    fn set_hl(&mut self, word: Word) {
        self.set_register16(Register16::HL, word);
    }

    pub fn get_flag(&self, flag: Byte) -> bool {
        assert_eq!(flag.count_ones(), 1);
        (self.f & flag) > 0
    }

    fn set_flag(&mut self, flag: Byte) {
        assert_eq!(flag.count_ones(), 1);
        self.f |= flag;
    }

    fn reset_flag(&mut self, flag: Byte) {
        assert_eq!(flag.count_ones(), 1);
        self.f &= !flag;
    }

    fn reset_all_flags(&mut self) {
        self.f = 0;
    }

    fn zero_flag(&mut self, result: Byte) {
        self.reset_flag(Self::ZERO_FLAG);
        if result == 0 {
            self.set_flag(Self::ZERO_FLAG);
        }
    }

    fn half_carry_flag_add(&mut self, b1: Byte, b2: Byte) {
        self.reset_flag(Self::HALF_CARRY_FLAG);
        if (b1 & 0xF) + (b2 & 0xF) > 0x0F {
            self.set_flag(Self::HALF_CARRY_FLAG);
        }
    }

    fn half_carry_flag_add16(&mut self, w1: Word, w2: Word) {
        self.reset_flag(Self::HALF_CARRY_FLAG);
        if (w1 & 0xFFF) + (w2 & 0xFFF) > 0xFFF {
            self.set_flag(Self::HALF_CARRY_FLAG);
        }
    }

    fn half_carry_flag_adc(&mut self, b1: Byte, b2: Byte, cf: Byte) {
        assert!(cf <= 1);
        self.reset_flag(Self::HALF_CARRY_FLAG);
        if (b1 & 0xF) + (b2 & 0xF) + cf > 0x0F {
            self.set_flag(Self::HALF_CARRY_FLAG);
        }
    }

    fn half_carry_flag_sub(&mut self, b1: Byte, b2: Byte) {
        self.reset_flag(Self::HALF_CARRY_FLAG);
        if (b1 & 0x0F) < (b2 & 0x0F) {
            self.set_flag(Self::HALF_CARRY_FLAG);
        }
    }

    fn half_carry_flag_sbc(&mut self, b1: Byte, b2: Byte, cf: Byte) {
        assert!(cf <= 1);
        self.reset_flag(Self::HALF_CARRY_FLAG);
        if (b1 & 0x0F) < (b2 & 0x0F) + cf {
            self.set_flag(Self::HALF_CARRY_FLAG);
        }
    }

    fn get_register(&self, reg: Register) -> Byte {
        match reg {
            Register::A => self.a,
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::HL => panic!("Unknown register HL"),
        }
    }

    fn set_register(&mut self, reg: Register, byte: Byte) {
        match reg {
            Register::A => self.a = byte,
            Register::B => self.b = byte,
            Register::C => self.c = byte,
            Register::D => self.d = byte,
            Register::E => self.e = byte,
            Register::H => self.h = byte,
            Register::L => self.l = byte,
            Register::HL => panic!("Unknown register HL"),
        }
    }

    fn get_register16(&self, reg: Register16) -> Word {
        match reg {
            Register16::SP => self.sp,
            Register16::BC => bytes2word(self.c, self.b),
            Register16::DE => bytes2word(self.e, self.d),
            Register16::AF => bytes2word(self.f, self.a),
            Register16::HL => bytes2word(self.l, self.h),
        }
    }

    fn set_register16(&mut self, reg: Register16, word: Word) {
        match reg {
            Register16::SP => self.sp = word,
            Register16::BC => {
                self.b = word.get_high();
                self.c = word.get_low();
            }
            Register16::DE => {
                self.d = word.get_high();
                self.e = word.get_low();
            }
            Register16::AF => {
                self.a = word.get_high();
                self.f = word.get_low();
                self.f &= 0xf0;
            }
            Register16::HL => {
                self.h = word.get_high();
                self.l = word.get_low();
            }
        }
    }

    fn get_condition(&self, cc: Condition) -> bool {
        match cc {
            Condition::NonZero => !self.get_flag(Self::ZERO_FLAG),
            Condition::Zero => self.get_flag(Self::ZERO_FLAG),
            Condition::NotCarry => !self.get_flag(Self::CARRY_FLAG),
            Condition::Carry => self.get_flag(Self::CARRY_FLAG),
        }
    }

    /// Push pc register values to [sp-1],[sp-2]
    fn push_pc_stack(&mut self, memory: &mut Memory) {
        self.sp -= 1;
        memory.write_byte(self.sp, self.pc.get_high());
        self.sp -= 1;
        memory.write_byte(self.sp, self.pc.get_low());
    }

    /// Pop pc register values from [sp+1],[sp+2]
    fn pop_pc_stack(&mut self, memory: &mut Memory) {
        let lsb = memory.read_byte(self.sp);
        self.sp += 1;
        let msb = memory.read_byte(self.sp);
        self.sp += 1;
        self.pc = bytes2word(lsb, msb);
    }

    /// Get ime flag
    fn get_ime(&self) -> bool {
        self.ime.1
    }

    /// Enable the ime flag
    fn ime_enable(&mut self) {
        if self.ime.0.is_none() {
            self.ime.0 = Some(2);
        }
    }

    /// Enable ime flag no delay
    fn ime_enable_no_delay(&mut self) {
        self.ime.1 = true;
    }

    /// Disable the ime flag
    fn ime_disable(&mut self) {
        self.ime = (None, false);
    }

    /// Step the ime delay
    pub fn ime_step(&mut self) {
        if let Some(mut delay) = self.ime.0 {
            delay -= 1;
            if delay == 0 {
                self.ime = (None, true);
            } else {
                self.ime.0 = Some(delay);
            }
        }
    }

    pub fn display_registers(&self, to_debug: bool) {
        if to_debug {
            debug!("Registers:");
            debug!(
                "\tA: {:#04X?}\tF: {:#04X?}\tB: {:#04X?}\tC: {:#04X?}",
                self.a, self.f, self.b, self.c,
            );
            debug!(
                "\tD: {:#04X?}\tE: {:#04X?}\tH: {:#04X?}\tL: {:#04X?}",
                self.d, self.e, self.h, self.l
            );
            debug!("\tSP: {:#06X?}\tPC: {:#06X}", self.sp, self.pc);
            debug!(
                "\tIME: {}\t Flags: {}",
                if self.ime.1 { "ENABLED" } else { "DISABLED" },
                self.display_flags()
            );
        } else {
            info!("Registers:");
            info!(
                "\tA: {:#04X?}\tF: {:#04X?}\tB: {:#04X?}\tC: {:#04X?}",
                self.a, self.f, self.b, self.c,
            );
            info!(
                "\tD: {:#04X?}\tE: {:#04X?}\tH: {:#04X?}\tL: {:#04X?}",
                self.d, self.e, self.h, self.l
            );
            info!("\tSP: {:#06X?}\tPC: {:#06X}", self.sp, self.pc);
            info!(
                "\tIME: {}\t Flags: {}",
                if self.ime.1 { "ENABLED" } else { "DISABLED" },
                self.display_flags()
            );
        }
    }

    fn display_flags(&self) -> String {
        format!(
            "{}{}{}{}",
            if self.get_flag(Self::CARRY_FLAG) {
                "C"
            } else {
                "-"
            },
            if self.get_flag(Self::HALF_CARRY_FLAG) {
                "H"
            } else {
                "-"
            },
            if self.get_flag(Self::SUBTRACT_FLAG) {
                "N"
            } else {
                "-"
            },
            if self.get_flag(Self::ZERO_FLAG) {
                "Z"
            } else {
                "-"
            },
        )
    }
}
