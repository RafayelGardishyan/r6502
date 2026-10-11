#[cfg(test)]
mod test {
    use crate::r6502::instructions::*;
    use crate::r6502::cpu::{StatusFlags, CPU, MEM, Byte};

    #[test]
    fn lda_imm_sets_a_to_immediate() {
        let (mut mem, mut cpu) = setup();

        // Given
        const EXPECTED_CYCLES: i32 = 2; // for LDA_IMM
        const VAL: Byte = 0x42;


        mem[0xFFFC] = LDA_IMM; // A <- imm
        mem[0xFFFD] = VAL; // Value to read


        let cpu_copy = cpu;

        // When
        let cycles_taken = cpu.execute(EXPECTED_CYCLES, &mut mem);

        // Then
        assert_eq!(cpu.a, 0x42);
        assert_eq!(cycles_taken, EXPECTED_CYCLES);
        check_status_flags_load(&cpu, &cpu_copy, VAL);
    }

    #[test]
    fn lda_zp_sets_a_to_address_in_zero_page() {
        let (mut mem, mut cpu) = setup();

        // Given
        const VAL: Byte = 0x69;
        const EXPECTED_CYCLES: i32 = 3; // for LDA_ZP

        mem[0xFFFC] = LDA_ZP; // A <- imm
        mem[0xFFFD] = 0x42; // Address to read from
        mem[0x0042] = VAL; // Value to read


        let cpu_copy = cpu;

        // When
        let cycles_taken = cpu.execute(EXPECTED_CYCLES, &mut mem);

        // Then
        assert_eq!(cpu.a, 0x69);
        assert_eq!(cycles_taken, EXPECTED_CYCLES);
        check_status_flags_load(&cpu, &cpu_copy, VAL);
    }

    fn check_status_flags_load(cpu: &CPU, cpu_copy: &CPU, val: Byte) {
        // These are affected by the load
        assert_eq!(cpu.sf.get(StatusFlags::NEGATIVE),  (((1 as Byte) << 7) & val) == 1);
        assert_eq!(cpu.sf.get(StatusFlags::ZERO),      val == 0);
        // Rest is untouched
        assert_eq!(cpu.sf.get(StatusFlags::BREAK),     cpu_copy.sf.get(StatusFlags::BREAK));
        assert_eq!(cpu.sf.get(StatusFlags::CARRY),     cpu_copy.sf.get(StatusFlags::BREAK));
        assert_eq!(cpu.sf.get(StatusFlags::DECIMAL),   cpu_copy.sf.get(StatusFlags::BREAK));
        assert_eq!(cpu.sf.get(StatusFlags::INTERRUPT), cpu_copy.sf.get(StatusFlags::BREAK));
        assert_eq!(cpu.sf.get(StatusFlags::OVERFLOW),  cpu_copy.sf.get(StatusFlags::BREAK));
        assert_eq!(cpu.sf.get(StatusFlags::UNUSED),    cpu_copy.sf.get(StatusFlags::BREAK));
    }

    fn setup() -> (MEM, CPU) {
        let mut mem = MEM::default();
        let mut cpu = CPU::default();

        cpu.reset(&mut mem);
        (mem, cpu)
    }
}