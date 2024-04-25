#[cfg(test)]
mod tests {
    use crate::clock::Clock;
    use crate::cpu::{
        Condition, Instruction, Register, Register16, SizedInstruction, CARRY_FLAG, CPU,
        HALF_CARRY_FLAG, SUBTRACT_FLAG, ZERO_FLAG,
    };
    use crate::memory::Memory;

    #[test]
    fn decode_ldrr() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x41]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_R_R(Register::B, Register::C),
                size: 1
            }
        )
    }

    #[test]
    fn decode_ldrn() {
        let mut memory = Memory::new();

        let n = 3;
        memory.write_test(vec![0x06, n]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_R_N(Register::B, n),
                size: 2
            }
        )
    }

    #[test]
    fn decode_ldrhl() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x46]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_R_HL(Register::B),
                size: 1
            }
        )
    }

    #[test]
    fn decode_ldhlr() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x70]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_HL_R(Register::B),
                size: 1
            }
        )
    }

    #[test]
    fn decode_ldhln() {
        let mut memory = Memory::new();

        let n = 3;
        memory.write_test(vec![0x36, n]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_HL_N(n),
                size: 2
            }
        )
    }

    #[test]
    fn decode_ldabc() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x0A]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_A_BC,
                size: 1
            }
        )
    }

    #[test]
    fn decode_ldade() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x1A]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_A_DE,
                size: 1
            }
        )
    }

    #[test]
    fn decode_ldbca() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x02]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_BC_A,
                size: 1
            }
        )
    }

    #[test]
    fn decode_lddea() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x12]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_DE_A,
                size: 1
            }
        )
    }

    #[test]
    fn decode_ldann() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xFA, 0x20, 0x03]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_A_NN(0x0320),
                size: 3
            }
        )
    }

    #[test]
    fn decode_ldnna() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xEA, 0x20, 0x03]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_NN_A(0x0320),
                size: 3
            }
        )
    }

    #[test]
    fn decode_ldhac() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xf2]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LDH_A_C,
                size: 1
            }
        )
    }

    #[test]
    fn decode_ldhca() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xe2]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LDH_C_A,
                size: 1
            }
        )
    }

    #[test]
    fn decode_ldhan() {
        let mut memory = Memory::new();

        let n = 10;
        memory.write_test(vec![0xf0, n]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LDH_A_N(n),
                size: 2
            }
        )
    }

    #[test]
    fn decode_ldhna() {
        let mut memory = Memory::new();

        let n = 10;
        memory.write_test(vec![0xe0, n]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LDH_N_A(n),
                size: 2
            }
        )
    }

    #[test]
    fn decode_ldahld() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x3a]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_A_HL_D,
                size: 1
            }
        )
    }

    #[test]
    fn decode_ldhdad() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x32]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_HL_A_D,
                size: 1
            }
        )
    }

    #[test]
    fn decode_ldahli() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x2a]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_A_HL_I,
                size: 1
            }
        )
    }

    #[test]
    fn decode_ldhlai() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x22]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_HL_A_I,
                size: 1
            }
        )
    }

    #[test]
    fn decode_ldrrnn() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x01, 0x10, 0x20]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_RR_NN(Register16::BC, 0x2010),
                size: 3
            }
        )
    }

    #[test]
    fn decode_ldspnn() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x31, 0x10, 0x20]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_RR_NN(Register16::SP, 0x2010),
                size: 3
            }
        )
    }

    #[test]
    fn decode_ldnnsp() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x08, 0x30, 0x20]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_NN_SP(0x2030),
                size: 3
            }
        )
    }

    #[test]
    fn decode_ldsphl() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xf9]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_SP_HL,
                size: 1
            }
        )
    }

    #[test]
    fn decode_ldhlsp() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xF8, 0xFF]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::LD_HL_SP(-1),
                size: 2
            }
        )
    }

    #[test]
    fn decode_push() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xC5]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::PUSH(Register16::BC),
                size: 1
            }
        )
    }

    #[test]
    fn decode_pop() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xC1]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::POP(Register16::BC),
                size: 1
            }
        )
    }

    #[test]
    fn decode_addr() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x80]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::ADD_R(Register::B),
                size: 1
            }
        )
    }

    #[test]
    fn decode_addhl() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x86]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::ADD_HL,
                size: 1
            }
        )
    }

    #[test]
    fn decode_addn() {
        let mut memory = Memory::new();

        let n = 0xf0;

        memory.write_test(vec![0xC6, n]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::ADD_N(n),
                size: 2
            }
        )
    }

    #[test]
    fn decode_adcr() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x88]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::ADC_R(Register::B),
                size: 1
            }
        )
    }

    #[test]
    fn decode_adchl() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x8E]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::ADC_HL,
                size: 1
            }
        )
    }

    #[test]
    fn decode_adcn() {
        let mut memory = Memory::new();

        let n = 0xf0;

        memory.write_test(vec![0xCE, n]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::ADC_N(n),
                size: 2
            }
        )
    }

    #[test]
    fn decode_subr() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x90]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::SUB_R(Register::B),
                size: 1
            }
        )
    }

    #[test]
    fn decode_subhl() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x96]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::SUB_HL,
                size: 1
            }
        )
    }

    #[test]
    fn decode_subn() {
        let mut memory = Memory::new();

        let n = 0x10;
        memory.write_test(vec![0xD6, n]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::SUB_N(n),
                size: 2
            }
        )
    }

    #[test]
    fn decode_sbcr() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x98]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::SBC_R(Register::B),
                size: 1
            }
        )
    }

    #[test]
    fn decode_sbchl() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x9E]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::SBC_HL,
                size: 1
            }
        )
    }

    #[test]
    fn decode_sbcn() {
        let mut memory = Memory::new();

        let n = 0x10;
        memory.write_test(vec![0xDE, n]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::SBC_N(n),
                size: 2
            }
        )
    }

    #[test]
    fn decode_cpr() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xB8]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::CP_R(Register::B),
                size: 1
            }
        )
    }

    #[test]
    fn decode_cphl() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xBE]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::CP_HL,
                size: 1
            }
        )
    }

    #[test]
    fn decode_cpn() {
        let mut memory = Memory::new();

        let n = 100;
        memory.write_test(vec![0xFE, n]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::CP_N(n),
                size: 2
            }
        )
    }

    #[test]
    fn decode_incr() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x04]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::INC_R(Register::B),
                size: 1
            }
        )
    }

    #[test]
    fn decode_inchl() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x34]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::INC_HL,
                size: 1
            }
        )
    }

    #[test]
    fn decode_decr() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x05]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::DEC_R(Register::B),
                size: 1
            }
        )
    }

    #[test]
    fn decode_dechl() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x35]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::DEC_HL,
                size: 1
            }
        )
    }

    #[test]
    fn decode_andr() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xA0]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::AND_R(Register::B),
                size: 1
            }
        )
    }

    #[test]
    fn decode_andhl() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xA6]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::AND_HL,
                size: 1
            }
        )
    }

    #[test]
    fn decode_andn() {
        let mut memory = Memory::new();

        let n = 100;
        memory.write_test(vec![0xE6, n]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::AND_N(n),
                size: 2
            }
        )
    }

    #[test]
    fn decode_orr() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xB0]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::OR_R(Register::B),
                size: 1
            }
        )
    }

    #[test]
    fn decode_orhl() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xB6]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::OR_HL,
                size: 1
            }
        )
    }

    #[test]
    fn decode_orn() {
        let mut memory = Memory::new();

        let n = 100;
        memory.write_test(vec![0xF6, n]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::OR_N(n),
                size: 2
            }
        )
    }

    #[test]
    fn decode_xorr() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xA8]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::XOR_R(Register::B),
                size: 1
            }
        )
    }

    #[test]
    fn decode_xorhl() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xAE]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::XOR_HL,
                size: 1
            }
        )
    }

    #[test]
    fn decode_xorn() {
        let mut memory = Memory::new();

        let n = 100;
        memory.write_test(vec![0xEE, n]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::XOR_N(n),
                size: 2
            }
        )
    }

    #[test]
    fn decode_ccf() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x3F]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::CCF,
                size: 1
            }
        )
    }

    #[test]
    fn decode_scf() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x37]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::SCF,
                size: 1
            }
        )
    }

    #[test]
    fn decode_daa() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x27]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::DAA,
                size: 1
            }
        )
    }

    #[test]
    fn decode_incrr() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x03]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::INC_RR(Register16::BC),
                size: 1
            }
        )
    }

    #[test]
    fn decode_decrr() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x0B]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::DEC_RR(Register16::BC),
                size: 1
            }
        )
    }

    #[test]
    fn decode_jpnn() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xC3, 0x20, 0x30]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::JP_NN(0x3020),
                size: 3
            }
        )
    }

    #[test]
    fn decode_jphl() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xE9]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::JP_HL,
                size: 1
            }
        )
    }

    #[test]
    fn decode_jpccnn() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xC2, 0x20, 0x30]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::JP_CC_NN(Condition::NonZero, 0x3020),
                size: 3
            }
        )
    }

    #[test]
    fn decode_jr() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x18, 0xff]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::JR(-1),
                size: 2
            }
        )
    }

    #[test]
    fn decode_jrz() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x28, 0xff]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::JR_CC(Condition::Zero, -1),
                size: 2
            }
        )
    }

    #[test]
    fn decode_jrcc() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x20, 0xff]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::JR_CC(Condition::NonZero, -1),
                size: 2
            }
        )
    }

    #[test]
    fn decode_callnn() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xCD, 0xff, 0x10]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::CALL(0x10ff),
                size: 3
            }
        )
    }

    #[test]
    fn decode_callccnn() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xC4, 0xff, 0x10]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::CALL_CC(Condition::NonZero, 0x10ff),
                size: 3
            }
        )
    }

    #[test]
    fn decode_ret() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xC9]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::RET,
                size: 1
            }
        )
    }

    #[test]
    fn decode_retcc() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xC0]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::RET_CC(Condition::NonZero),
                size: 1
            }
        )
    }

    #[test]
    fn decode_reti() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xD9]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::RETI,
                size: 1
            }
        )
    }

    #[test]
    fn decode_rst() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xDF]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::RST(0x18),
                size: 1
            }
        )
    }

    #[test]
    fn decode_addhlrr() {
        let mut memory = Memory::new();

        let e = 255;
        memory.write_test(vec![0xE8, e]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::ADD_SP_E(-1),
                size: 2
            }
        )
    }

    #[test]
    fn decode_addspe() {
        let mut memory = Memory::new();

        let e = 255;
        memory.write_test(vec![0xE8, e]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::ADD_SP_E(-1),
                size: 2
            }
        )
    }

    #[test]
    fn decode_rra() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x1F]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::RRA,
                size: 1
            }
        )
    }

    #[test]
    fn decode_rrca() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x0F]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::RRCA,
                size: 1
            }
        )
    }

    #[test]
    fn decode_rla() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x17]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::RLA,
                size: 1
            }
        )
    }

    #[test]
    fn decode_rlca() {
        let mut memory = Memory::new();

        memory.write_test(vec![0x07]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::RLCA,
                size: 1
            }
        )
    }

    #[test]
    fn decode_rlc() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xCB, 0x01]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::RLC(Register::C),
                size: 2
            }
        )
    }

    #[test]
    fn decode_rl() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xCB, 0x12]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::RL(Register::D),
                size: 2
            }
        )
    }

    #[test]
    fn decode_sla() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xCB, 0x24]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::SLA(Register::H),
                size: 2
            }
        )
    }

    #[test]
    fn decode_slahl() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xCB, 0x26]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::SLA_HL,
                size: 2
            }
        )
    }

    #[test]
    fn decode_swap() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xCB, 0x35]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::SWAP(Register::L),
                size: 2
            }
        )
    }

    #[test]
    fn decode_rrc() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xCB, 0x08]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::RRC(Register::B),
                size: 2
            }
        )
    }

    #[test]
    fn decode_rr() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xCB, 0x1b]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::RR(Register::E),
                size: 2
            }
        )
    }

    #[test]
    fn decode_sra() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xCB, 0x2c]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::SRA(Register::H),
                size: 2
            }
        )
    }

    #[test]
    fn decode_srl() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xCB, 0x3f]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::SRL(Register::A),
                size: 2
            }
        )
    }

    #[test]
    fn decode_bit() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xCB, 0x62]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::BIT(4, Register::D),
                size: 2
            }
        )
    }

    #[test]
    fn decode_res() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xCB, 0x99]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::RES(3, Register::C),
                size: 2
            }
        )
    }

    #[test]
    fn decode_set() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xCB, 0xea]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::SET(5, Register::D),
                size: 2
            }
        )
    }

    #[test]
    fn decode_ei() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xFB]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::EI,
                size: 1
            }
        )
    }

    #[test]
    fn decode_di() {
        let mut memory = Memory::new();

        memory.write_test(vec![0xF3]);

        let instr = SizedInstruction::decode(&mut memory, 0).unwrap();
        assert_eq!(
            instr,
            SizedInstruction {
                instruction: Instruction::DI,
                size: 1
            }
        )
    }

    #[test]
    fn execute_addr() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();
        let mut clock = Clock::new();

        cpu.pc = 0;
        memory.write_test(vec![0x80]);

        // Set initial register values
        cpu.a = 0x10;
        cpu.b = 0x20;

        // Execute ADD instruction
        cpu.execute(&mut memory, &mut clock);

        assert_eq!(cpu.a, 0x30);
        assert_eq!(cpu.b, 0x20);
    }

    #[test]
    fn execute_addhl() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();
        let mut clock = Clock::new();

        memory.write_test(vec![0x86]);

        cpu.a = 0x10;
        cpu.h = 0x12;
        cpu.l = 0x34;

        memory.write_byte(0x1234, 0x20);

        cpu.execute(&mut memory, &mut clock);

        assert_eq!(cpu.h, 0x12);
        assert_eq!(cpu.l, 0x34);
        assert_eq!(memory.read_byte(cpu.get_hl()), 0x20);
        assert_eq!(cpu.a, 0x30);
    }

    #[test]
    fn execute_addn() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();
        let mut clock = Clock::new();

        memory.write_test(vec![0xC6, 0x20]);

        cpu.a = 0x10;

        cpu.execute(&mut memory, &mut clock);

        assert_eq!(cpu.a, 0x30);
    }

    #[test]
    fn execute_xor() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();
        let mut clock = Clock::new();

        memory.write_test(vec![0xA8]);

        cpu.a = 0b11001100;
        cpu.b = 0b10101010;

        cpu.execute(&mut memory, &mut clock);

        assert_eq!(cpu.a, 0b01100110);
    }

    #[test]
    fn execute_addspe() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();
        let mut clock = Clock::new();

        memory.write_test(vec![0xE8, 0xfe]);

        cpu.sp = 1;

        cpu.execute(&mut memory, &mut clock);

        assert_eq!(cpu.sp, 0xffff);
        assert_eq!(cpu.get_flag(HALF_CARRY_FLAG), false);
        assert_eq!(cpu.get_flag(CARRY_FLAG), false);
    }

    #[test]
    fn execute_addspe_hc() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();
        let mut clock = Clock::new();

        memory.write_test(vec![0xE8, 0xff]);

        cpu.sp = 0xf;

        cpu.execute(&mut memory, &mut clock);

        assert_eq!(cpu.sp, 0xe);
        assert_eq!(cpu.get_flag(HALF_CARRY_FLAG), true);
        assert_eq!(cpu.get_flag(CARRY_FLAG), true);
    }

    #[test]
    fn execute_swap() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();
        let mut clock = Clock::new();

        memory.write_test(vec![0xCB, 0x30]);

        cpu.b = 0xef;

        cpu.execute(&mut memory, &mut clock);

        assert_eq!(cpu.b, 0xfe);
        assert_eq!(cpu.get_flag(ZERO_FLAG), false);
        assert_eq!(cpu.get_flag(HALF_CARRY_FLAG), false);
        assert_eq!(cpu.get_flag(CARRY_FLAG), false);
        assert_eq!(cpu.get_flag(SUBTRACT_FLAG), false);
    }

    #[test]
    fn execute_swap_zero() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();
        let mut clock = Clock::new();

        memory.write_test(vec![0xCB, 0x30]);

        cpu.b = 0;

        cpu.execute(&mut memory, &mut clock);

        assert_eq!(cpu.b, 0);
        assert_eq!(cpu.get_flag(ZERO_FLAG), true);
        assert_eq!(cpu.get_flag(HALF_CARRY_FLAG), false);
        assert_eq!(cpu.get_flag(CARRY_FLAG), false);
        assert_eq!(cpu.get_flag(SUBTRACT_FLAG), false);
    }

    #[test]
    fn execute_ldhlsp() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();
        let mut clock = Clock::new();

        memory.write_test(vec![0xF8, 0xFE]);

        cpu.sp = 0x2;

        cpu.execute(&mut memory, &mut clock);

        assert_eq!(cpu.get_hl(), 0);
        assert_eq!(cpu.get_flag(HALF_CARRY_FLAG), true);
        assert_eq!(cpu.get_flag(CARRY_FLAG), true);
        assert_eq!(cpu.get_flag(ZERO_FLAG), false);
    }

    #[test]
    fn execute_cpl() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();
        let mut clock = Clock::new();

        memory.write_test(vec![0x2F]);

        cpu.a = 0xe2;

        cpu.execute(&mut memory, &mut clock);

        assert_eq!(cpu.a, 0x1d);
    }

    #[test]
    fn execute_set() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();
        let mut clock = Clock::new();

        memory.write_test(vec![0xCB, 0xC0]);

        cpu.b = 0xCA;

        cpu.execute(&mut memory, &mut clock);

        assert_eq!(cpu.b, 0xCB);

        let mut cpu = CPU::new();
        let mut memory = Memory::new();
        let mut clock = Clock::new();

        memory.write_test(vec![0xCB, 0xC0]);

        cpu.b = 0xCB;

        cpu.execute(&mut memory, &mut clock);

        assert_eq!(cpu.b, 0xCB);
    }

    #[test]
    fn execute_res() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();
        let mut clock = Clock::new();

        memory.write_test(vec![0xCB, 0x80]);

        cpu.b = 0xCB;

        cpu.execute(&mut memory, &mut clock);

        assert_eq!(cpu.b, 0xCA);

        let mut cpu = CPU::new();
        let mut memory = Memory::new();
        let mut clock = Clock::new();

        memory.write_test(vec![0xCB, 0x80]);

        cpu.b = 0xCA;

        cpu.execute(&mut memory, &mut clock);

        assert_eq!(cpu.b, 0xCA);
    }
}
