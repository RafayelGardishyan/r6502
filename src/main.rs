use crate::r6502::cpu::{CPU, MEM};
use crate::r6502::instructions::*;

pub mod r6502;
mod test;

fn main() {
    let mut mem = MEM::default();
    let mut cpu = CPU::default();

    cpu.reset(&mut mem);

    // Sample program
    mem[0xFFFC] = LDA_ABX; // A <- Absolute addr + X
    mem[0xFFFD] = 0x41; // Address to read from
    mem[0xFFFE] = 0x69; // Address to read from
    mem[0x6942] = 0x67; // Value to be read

    cpu.x = 0x01;

    let cycles_taken = cpu.execute(1, &mut mem);

    println!("Took {} cycles", cycles_taken);
    println!("Value of CPU.A = 0x{:x}", cpu.a);

}
