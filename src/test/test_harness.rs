use crate::r6502::cpu::{MockRAM, CPU, MAX_MEMORY, Bus, Word, Byte};
use super::test_loader::*;

fn compare_cpu(cpu: CPU, cpu_expected: CPU) {
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

fn compare_memory(mem: MockRAM, mem_expected: MockRAM) {
    for i in 0..MAX_MEMORY {
        assert_eq!(mem.read(i as Word), mem_expected.read(i as Word),
               "MEM mismatch: address={:#06X} expected {:#04X}, got {:#04X}",
               i, mem_expected.read(i as Word), mem.read(i as Word));
    }
}

fn run_single_test(test_case: JsonTestCase) {
    let (mut cpu, mut mem) = create_cpu_from_state(test_case.initial);
    let (final_cpu, final_mem) = create_cpu_from_state(test_case.final_state);

    let expected_cycles = test_case.cycles.len();

    let cycles_taken = cpu.execute(expected_cycles, &mut mem);

    compare_cpu(cpu, final_cpu);
    compare_memory(mem, final_mem);
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
        run_single_test(test_case)
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
}