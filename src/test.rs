#[cfg(test)]
mod tests {
    use crate::cpu::{Instruction, Register, Register16, SizedInstruction, CPU};
    use crate::memory::Memory;

    #[test]
    fn test_decode_ldrr() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0x41]);

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
    fn test_decode_ldrn() {
        let mut memory = Memory::new();

        let n = 3;
        memory.load_rom(vec![0x06, n]);

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
    fn test_decode_ldrhl() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0x46]);

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
    fn test_decode_ldhlr() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0x70]);

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
    fn test_decode_ldhln() {
        let mut memory = Memory::new();

        let n = 3;
        memory.load_rom(vec![0x36, n]);

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
    fn test_decode_ldabc() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0x0A]);

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
    fn test_decode_ldade() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0x1A]);

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
    fn test_decode_ldbca() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0x02]);

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
    fn test_decode_lddea() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0x12]);

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
    fn test_decode_ldann() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0xFA, 0x20, 0x03]);

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
    fn test_decode_ldnna() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0xEA, 0x20, 0x03]);

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
    fn test_decode_ldhac() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0xf2]);

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
    fn test_decode_ldhca() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0xe2]);

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
    fn test_decode_ldhan() {
        let mut memory = Memory::new();

        let n = 10;
        memory.load_rom(vec![0xf0, n]);

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
    fn test_decode_ldhna() {
        let mut memory = Memory::new();

        let n = 10;
        memory.load_rom(vec![0xe0, n]);

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
    fn test_decode_ldahld() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0x3a]);

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
    fn test_decode_ldhdad() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0x32]);

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
    fn test_decode_ldahli() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0x2a]);

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
    fn test_decode_ldhlai() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0x22]);

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
    fn test_decode_ldrrnn() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0x01, 0x10, 0x20]);

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
    fn test_decode_ldnnsp() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0x08, 0x30, 0x20]);

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
    fn test_decode_ldsphl() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0xf9]);

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
    fn test_decode_ldhlsp() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0xF8, 0xFF]);

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
    fn test_decode_push() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0xC5]);

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
    fn test_decode_pop() {
        let mut memory = Memory::new();

        memory.load_rom(vec![0xC1]);

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
    fn test_addr_instruction() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        memory.load_rom(vec![0x80]);

        // Set initial register values
        cpu.a = 0x10;
        cpu.b = 0x20;

        // Execute ADD instruction
        cpu.execute(&mut memory);

        assert_eq!(cpu.a, 0x30);
        assert_eq!(cpu.b, 0x20);
    }

    #[test]
    fn test_add_hl_instruction() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        memory.load_rom(vec![0x86]);

        cpu.a = 0x10;
        cpu.h = 0x12;
        cpu.l = 0x34;

        memory.write_byte(0x1234, 0x20);

        cpu.execute(&mut memory);

        assert_eq!(cpu.h, 0x12);
        assert_eq!(cpu.l, 0x34);
        assert_eq!(cpu.a, 0x30);
    }

    #[test]
    fn test_add_n_instruction() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        memory.load_rom(vec![0xC6, 0x20]);

        cpu.a = 0x10;

        cpu.execute(&mut memory);

        assert_eq!(cpu.a, 0x30);
    }

    #[test]
    fn test_xor_instruction() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new();

        memory.load_rom(vec![0xA8]);

        cpu.a = 0b11001100;
        cpu.b = 0b10101010;

        cpu.execute(&mut memory);

        assert_eq!(cpu.a, 0b01100110);
    }
}
