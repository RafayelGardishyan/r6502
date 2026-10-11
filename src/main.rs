use crate::r6502::cpu::{CPU, MEM};
use crate::r6502::instructions::LDA_ZP;

pub mod r6502;
mod test;

fn main() {
    let mut mem = MEM::default();
    let mut cpu = CPU::default();

    cpu.reset(&mut mem);

    // Sample program
    mem[0xFFFC] = LDA_ZP; // A <- ZP Address
    mem[0xFFFD] = 0x42; // Address to read from
    mem[0x0042] = 0x69; // Value to be read

    let cycles_taken = cpu.execute(3, &mut mem);

    println!("Took {} cycles", cycles_taken);
    println!("Value of CPU.A = 0x{:x}", cpu.a);

}
