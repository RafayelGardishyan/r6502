use crate::r6502::cpu::{CPU, MEM};

pub mod r6502;

fn main() {
    let mut mem = MEM::default();
    let mut cpu = CPU::default();

    cpu.reset(&mut mem);

    // Sample program
    mem[0xFFFC] = 0xA9;
    mem[0xFFFD] = 0x42;

    let cycles_taken = cpu.execute(2, &mut mem);

    println!("Took {} cycles", cycles_taken);
    println!("Value of CPU.A = 0x{:x}", cpu.a);

}
