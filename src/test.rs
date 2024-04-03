#[cfg(test)]
mod tests {
    use crate::cpu::CPU;
    use crate::memory::Memory;
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
}
