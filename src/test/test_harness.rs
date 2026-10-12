use crate::r6502::cpu::{MockRAM, CPU, MAX_MEMORY, Bus, Word, Byte};
use super::test_loader::*;

fn compare_cpu(&cpu: &CPU, &cpu_expected: &CPU) {
    assert_eq!(cpu.pc, cpu_expected.pc,
               "PC mismatch: expected {:#06X}, got {:#06X}",
               cpu_expected.pc, cpu.pc);
    assert_eq!(cpu.sp, cpu_expected.sp,
               "SP mismatch: expected {:#04X}, got {:#04X}",
               cpu_expected.sp, cpu.sp);
    assert_eq!(cpu.a,  cpu_expected.a,
               "A mismatch: expected {:#04X}, got {:#04X}",
               cpu_expected.a, cpu.a);
    assert_eq!(cpu.x,  cpu_expected.x,
               "X mismatch: expected {:#04X}, got {:#04X}",
               cpu_expected.x, cpu.x);
    assert_eq!(cpu.y,  cpu_expected.y,
               "Y mismatch: expected {:#04X}, got {:#04X}",
               cpu_expected.y, cpu.y);
    assert_eq!(cpu.sf, cpu_expected.sf,
               "SF mismatch");
}

fn compare_memory(mem: &MockRAM, mem_expected: &MockRAM) {
    if mem.data != mem_expected.data {
        // Make checks faster if they are correct
        for i in 0..MAX_MEMORY {
            let actual = mem.read(i as Word);
            let expected = mem_expected.read(i as Word);
            assert_eq!(
                actual, expected,
                "MEM mismatch: address={:#06X} expected {:#02X}, got {:#02X}",
                i, expected, actual
            );
        }
    }
}

fn run_single_test(test_case: &JsonTestCase) {
    let (mut cpu, mut mem) = create_cpu_from_state(&test_case.initial);
    let (final_cpu, final_mem) = create_cpu_from_state(&test_case.final_state);

    let expected_cycles = test_case.cycles.len();

    let cycles_taken = cpu.execute(expected_cycles, &mut mem);

    compare_cpu(&cpu, &final_cpu);
    compare_memory(&mem, &final_mem);
    assert_eq!(cycles_taken, expected_cycles,
    "Cycle mismatch: {} expected while {} were taken",
        expected_cycles, cycles_taken
    )


}

fn run_opcode_tests(opcode: Byte) {
    let test_cases = get_tests_for_instruction(opcode);
    let total = test_cases.len();

    for test_case in test_cases {
        let name = &test_case.name;
        println!("Running {}", name);
        run_single_test(&test_case)
    }

    println!("Opcode {:#04X}: passed all {} tests.", opcode, total);
}

#[cfg(test)]
mod json_tests {
    use crate::r6502::instructions::*;
    use super::*;

    #[test]
    fn test_lda_imm() {
        run_opcode_tests(LDA_IMM);
    }
    #[test]
    fn test_lda_zp() {
        run_opcode_tests(LDA_ZP);
    }
    #[test]
    fn test_lda_zpx() {
        run_opcode_tests(LDA_ZPX);
    }
    #[test]
    fn test_lda_abs() {
        run_opcode_tests(LDA_ABS);
    }
    #[test]
    fn test_lda_abx() {
        run_opcode_tests(LDA_ABX);
    }
    #[test]
    fn test_lda_aby() {
        run_opcode_tests(LDA_ABY);
    }
    #[test]
    fn test_lda_idx() {
        run_opcode_tests(LDA_IDX);
    }
    #[test]
    fn test_lda_idy() {
        run_opcode_tests(LDA_IDY);
    }

    // LDX
    #[test]
    fn test_ldx_imm() {
        run_opcode_tests(LDX_IMM);
    }

    #[test]
    fn test_ldx_zp() {
        run_opcode_tests(LDX_ZP);
    }
    #[test]
    fn test_ldx_zpy() {
        run_opcode_tests(LDX_ZPY);
    }
    #[test]
    fn test_ldx_abs() {
        run_opcode_tests(LDX_ABS);
    }
    #[test]
    fn test_ldx_aby() {
        run_opcode_tests(LDX_ABY);
    }

    // LDY
    #[test]
    fn test_ldy_imm() {
        run_opcode_tests(LDY_IMM);
    }

    #[test]
    fn test_ldy_zp() {
        run_opcode_tests(LDY_ZP);
    }
    #[test]
    fn test_ldy_zpy() {
        run_opcode_tests(LDY_ZPX);
    }
    #[test]
    fn test_ldy_abs() {
        run_opcode_tests(LDY_ABS);
    }
    #[test]
    fn test_ldy_abx() {
        run_opcode_tests(LDY_ABX);
    }

    // STA
    #[test]
    fn test_sta_zp() {
        run_opcode_tests(STA_ZP);
    }
    #[test]
    fn test_sta_zpx() {
        run_opcode_tests(STA_ZPX);
    }
    #[test]
    fn test_sta_abs() {
        run_opcode_tests(STA_ABS);
    }
    #[test]
    fn test_sta_abx() {
        run_opcode_tests(STA_ABX);
    }
    #[test]
    fn test_sta_aby() {
        run_opcode_tests(STA_ABY);
    }
    #[test]
    fn test_sta_idx() {
        run_opcode_tests(STA_IDX);
    }
    #[test]
    fn test_sta_idy() {
        run_opcode_tests(STA_IDY);
    }

    // STX
    #[test]
    fn test_stx_zp() {
        run_opcode_tests(STX_ZP);
    }
    #[test]
    fn test_stx_zpy() {
        run_opcode_tests(STX_ZPY);
    }
    #[test]
    fn test_stx_abs() {
        run_opcode_tests(STX_ABS);
    }

    // STY
    #[test]
    fn test_sty_zp() {
        run_opcode_tests(STY_ZP);
    }
    #[test]
    fn test_sty_zpx() {
        run_opcode_tests(STY_ZPX);
    }
    #[test]
    fn test_sty_abs() {
        run_opcode_tests(STY_ABS);
    }

    // Register Transfers (TAX TAY TXA TYA)
    #[test]
    fn test_tax() {
        run_opcode_tests(TAX);
    }
    #[test]
    fn test_tay() {
        run_opcode_tests(TAY);
    }
    #[test]
    fn test_txa() {
        run_opcode_tests(TXA);
    }
    #[test]
    fn test_tya() {
        run_opcode_tests(TYA);
    }
    #[test]
    fn test_tsx() {
        run_opcode_tests(TSX);
    }
    #[test]
    fn test_txs() {
        run_opcode_tests(TXS);
    }

    // Stack operations
    #[test]
    fn test_pha() {
        run_opcode_tests(PHA);
    }

    #[test]
    fn test_php() {
        run_opcode_tests(PHP);
    }

    #[test]
    fn test_pla() {
        run_opcode_tests(PLA);
    }

    #[test]
    fn test_plp() {
        run_opcode_tests(PLP);
    }

    // AND
    #[test]
    fn test_and_imm() {
        run_opcode_tests(AND_IMM);
    }
    #[test]
    fn test_and_zp() {
        run_opcode_tests(AND_ZP);
    }
    #[test]
    fn test_and_zpx() {
        run_opcode_tests(AND_ZPX);
    }
    #[test]
    fn test_and_abs() {
        run_opcode_tests(AND_ABS);
    }
    #[test]
    fn test_and_abx() {
        run_opcode_tests(AND_ABX);
    }
    #[test]
    fn test_and_aby() {
        run_opcode_tests(AND_ABY);
    }
    #[test]
    fn test_and_idx() {
        run_opcode_tests(AND_IDX);
    }
    #[test]
    fn test_and_idy() {
        run_opcode_tests(AND_IDY);
    }

    // EOR
    #[test]
    fn test_eor_imm() {
        run_opcode_tests(EOR_IMM);
    }
    #[test]
    fn test_eor_zp() {
        run_opcode_tests(EOR_ZP);
    }
    #[test]
    fn test_eor_zpx() {
        run_opcode_tests(EOR_ZPX);
    }
    #[test]
    fn test_eor_abs() {
        run_opcode_tests(EOR_ABS);
    }
    #[test]
    fn test_eor_abx() {
        run_opcode_tests(EOR_ABX);
    }
    #[test]
    fn test_eor_aby() {
        run_opcode_tests(EOR_ABY);
    }
    #[test]
    fn test_eor_idx() {
        run_opcode_tests(EOR_IDX);
    }
    #[test]
    fn test_eor_idy() {
        run_opcode_tests(EOR_IDY);
    }

    // ORA
    #[test]
    fn test_ora_imm() {
        run_opcode_tests(ORA_IMM);
    }
    #[test]
    fn test_ora_zp() {
        run_opcode_tests(ORA_ZP);
    }
    #[test]
    fn test_ora_zpx() {
        run_opcode_tests(ORA_ZPX);
    }
    #[test]
    fn test_ora_abs() {
        run_opcode_tests(ORA_ABS);
    }
    #[test]
    fn test_ora_abx() {
        run_opcode_tests(ORA_ABX);
    }
    #[test]
    fn test_ora_aby() {
        run_opcode_tests(ORA_ABY);
    }
    #[test]
    fn test_ora_idx() {
        run_opcode_tests(ORA_IDX);
    }
    #[test]
    fn test_ora_idy() {
        run_opcode_tests(ORA_IDY);
    }

    // BIT
    #[test]
    fn test_bit_zp() {
        run_opcode_tests(BIT_ZP);
    }
    #[test]
    fn test_bit_abs() {
        run_opcode_tests(BIT_ABS);
    }
}