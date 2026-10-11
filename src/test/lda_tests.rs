#[cfg(test)]
mod test {
    use crate::r6502::instructions::*;
    use crate::r6502::cpu::{StatusFlags, CPU, MockRAM, Byte, Bus};

    #[test]
    fn lda_imm_sets_a_to_immediate() {
        let (mut mem, mut cpu) = setup();

        // Given
        const EXPECTED_CYCLES: usize = 2; // for LDA_IMM
        const VAL: Byte = 0x42;


        mem.write(0xFFFC, LDA_IMM); // A <- imm
        mem.write(0xFFFD, VAL); // Value to read


        copy_run_check(&mut mem, &mut cpu, EXPECTED_CYCLES, VAL);
    }

    #[test]
    fn lda_zp_sets_a_to_address_in_zero_page() {
        let (mut mem, mut cpu) = setup();

        // Given
        const VAL: Byte = 0x69;
        const EXPECTED_CYCLES: usize = 3; // for LDA_ZP

        mem.write(0xFFFC, LDA_ZP); // A <- mem[ZP]
        mem.write(0xFFFD, 0x42); // Address to read from
        mem.write(0x0042, VAL); // Value to read


        copy_run_check(&mut mem, &mut cpu, EXPECTED_CYCLES, VAL);
    }

    #[test]
    fn lda_zpx_sets_a_to_address_plus_x_in_zero_page() {
        let (mut mem, mut cpu) = setup();

        // Given
        const VAL: Byte = 0x69;
        const EXPECTED_CYCLES: usize = 4; // for LDA_ZPX
        cpu.x = 0x04;

        mem.write(0xFFFC, LDA_ZPX); // A <- mem[ZP + X]
        mem.write(0xFFFD, 0x3E); // Address to read from
        mem.write(0x0042, VAL); // Value to read


        copy_run_check(&mut mem, &mut cpu, EXPECTED_CYCLES, VAL);
    }

    #[test]
    fn lda_imm_sets_a_to_absolute_memory_address() {
        let (mut mem, mut cpu) = setup();

        // Given
        const EXPECTED_CYCLES: usize = 4; // for LDA_ABS
        const VAL: Byte = 0x42;


        mem.write(0xFFFC, LDA_ABS); // A <- mem[abs]
        mem.write(0xFFFD, 0x69);     // Absolute address 0x6769 (Little endian)
        mem.write(0xFFFE, 0x67);
        mem.write(0x6769, VAL); // Value to read


        copy_run_check(&mut mem, &mut cpu, EXPECTED_CYCLES, VAL);
    }

    #[test]
    fn lda_does_not_change_other_registers() {
        let (mut mem, mut cpu) = setup();

        // Given
        const EXPECTED_CYCLES: usize = 2; // for LDA_IMM
        const VAL: Byte = 0x42;


        mem.write(0xFFFC, LDA_IMM); // A <- imm
        mem.write(0xFFFD, VAL); // Value to read


        let cpu_copy = cpu;

        // When
        cpu.execute(EXPECTED_CYCLES, &mut mem);

        // Then
        assert_eq!(cpu.a,  0x42);
        assert_eq!(cpu.x,  cpu_copy.x);
        assert_eq!(cpu.y,  cpu_copy.y);
        assert_eq!(cpu.sp, cpu_copy.sp);
        assert_eq!(cpu.pc, cpu_copy.pc + 2); // Instruction is 2 bytes
    }

    fn check_status_flags_load(cpu: &CPU, status_copy: &StatusFlags, val: Byte) {
        // These are affected by the load
        assert_eq!(cpu.sf.get(StatusFlags::NEGATIVE),  (((1 as Byte) << 7) & val) == 1);
        assert_eq!(cpu.sf.get(StatusFlags::ZERO),      val == 0);
        // Rest is untouched
        assert_eq!(cpu.sf.get(StatusFlags::BREAK),     status_copy.get(StatusFlags::BREAK));
        assert_eq!(cpu.sf.get(StatusFlags::CARRY),     status_copy.get(StatusFlags::BREAK));
        assert_eq!(cpu.sf.get(StatusFlags::DECIMAL),   status_copy.get(StatusFlags::BREAK));
        assert_eq!(cpu.sf.get(StatusFlags::INTERRUPT), status_copy.get(StatusFlags::BREAK));
        assert_eq!(cpu.sf.get(StatusFlags::OVERFLOW),  status_copy.get(StatusFlags::BREAK));
        assert_eq!(cpu.sf.get(StatusFlags::UNUSED),    status_copy.get(StatusFlags::BREAK));
    }

    fn copy_run_check(mem: &mut MockRAM, cpu: &mut CPU, expected_cycles: usize, expected_value: Byte) {
        let status_copy = cpu.sf;

        // When
        let cycles_taken = cpu.execute(expected_cycles, mem);

        // Then
        assert_eq!(cpu.a, expected_value);
        assert_eq!(cycles_taken, expected_cycles);
        check_status_flags_load(&cpu, &status_copy, expected_value);
    }

    fn setup() -> (MockRAM, CPU) {
        let mut mem = MockRAM::default();
        let mut cpu = CPU::default();

        cpu.reset(&mut mem);
        (mem, cpu)
    }
}