use std::fs;
use std::path::PathBuf;
use crate::r6502::cpu::{MockRAM, CPU, StatusFlags, Bus, Byte};

#[derive(serde::Deserialize)]
pub struct CpuState {
    pub pc: u16,
    pub s: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
    pub ram: Vec<(u16, u8)>,
}

#[derive(serde::Deserialize)]
pub struct JsonTestCase {
    pub name: String,
    pub initial: CpuState,

    #[serde(rename = "final")]
    pub final_state: CpuState,
    pub cycles: Vec<(u16, u8, String)>,
}

pub fn create_cpu_from_state(state: &CpuState) -> (CPU, MockRAM) {
    let mut cpu = CPU::default();
    let mut mem = MockRAM::default();

    cpu.reset(&mut mem);

    cpu.pc = state.pc;
    cpu.sp = state.s;
    cpu.a = state.a;
    cpu.x = state.x;
    cpu.y = state.y;
    cpu.sf = StatusFlags(state.p);

    for (addr, data) in &state.ram {
        mem.write(*addr, *data);
    }

    (cpu, mem)
}

pub fn get_tests_for_instruction(opcode: Byte) -> Vec<JsonTestCase> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let file_path = PathBuf::from(manifest_dir)
        .join("data")
        .join(format!("{:02x}.json", opcode));

    let file_data = fs::read_to_string(&file_path)
        .unwrap_or_else(|e| panic!("Unable to read {:?}: {}", file_path, e));

    serde_json::from_str(&file_data)
        .expect("Json is not properly formatted")
}