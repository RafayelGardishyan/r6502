use std::ops;
use std::ops::Index;
use std::process::exit;
use super::instructions;

pub type Byte = u8;
pub type Word = u16;

pub type Bit = bool;
pub const ZERO: Bit = false;
pub const ONE: Bit = true;

const MAX_MEMORY: usize = 1024 * 64; // 64 Kb

pub struct MEM {
    data: [Byte; MAX_MEMORY],
}

#[derive(Clone, Copy)]
pub struct StatusFlags(Byte);

#[derive(Copy, Clone)]
pub struct CPU {
    // Registers
    pub(crate) pc: Word, // Program counter
    pub(crate) sp: Byte, // Stack pointer
    pub(crate) a:  Byte, // Accumulator
    pub(crate) x:  Byte, // Index Register X
    pub(crate) y:  Byte, // Index Register Y
    pub(crate) sf: StatusFlags // Status Flags
}

pub enum Register {
    PC, SP, A, X, Y, NOREG
}

impl MEM {
    pub fn init(&mut self) {
        for i in 0..MAX_MEMORY {
            self.data[i] = 0;
        }
    }
}

impl Default for MEM {
    fn default() -> Self {
        MEM {
            data: [0; MAX_MEMORY]
        }
    }
}

impl Index<Word> for MEM {
    type Output = Byte;

    fn index(&self, index: Word) -> &Self::Output {
        &self.data[index as usize]
    }
}

impl ops::IndexMut<Word> for MEM {
    fn index_mut(&mut self, index: Word) -> &mut Self::Output {
        &mut self.data[index as usize]
    }
}

impl StatusFlags {
    pub const CARRY: u8     = 1 << 0;
    pub const ZERO: u8      = 1 << 1;
    pub const INTERRUPT: u8 = 1 << 2;
    pub const DECIMAL: u8   = 1 << 3;
    pub const BREAK: u8     = 1 << 4;
    pub const UNUSED: u8    = 1 << 5;
    pub const OVERFLOW: u8  = 1 << 6;
    pub const NEGATIVE: u8  = 1 << 7;

    pub fn set(&mut self, flag: u8, value: Bit) {
        if value { self.0 |= flag; } else { self.0 &= !flag; }
    }

    pub fn get(&self, flag: u8) -> Bit {
        (self.0 & flag) != 0
    }

    pub fn zero(&mut self) {
        self.0 &= 0
    }
}

impl Default for CPU {
    fn default() -> Self {
        CPU {
            pc: 0,
            sp: 0,
            a:  0,
            x:  0,
            y:  0,
            sf: StatusFlags(0)
        }
    }
}
impl CPU {
    pub fn reset(&mut self, mem: &mut MEM) {
        self.reset_with_address(0xFFFC, mem)
    }

    fn reset_with_address(&mut self, reset_vector: Word, mem: &mut MEM) {
        self.pc = reset_vector;
        self.sp = 0xFF;
        self.x = 0;
        self.y = 0;
        self.a = 0;
        self.sf.zero();
        mem.init();
    }

    pub fn execute(&mut self, cycles: i32, mem: &mut MEM) -> i32 {
        let mut cycles_remaining = cycles;

        while cycles_remaining > 0 {
            let (instruction, nc) = self.fetch_byte(cycles_remaining, &mem);
            cycles_remaining = nc;

            match instruction {
                // LDA
                instructions::LDA_IMM  =>
                    cycles_remaining = self.load_immediate(Register::A, cycles_remaining, mem),
                instructions::LDA_ZP =>
                    cycles_remaining = self.load_zero_page(Register::A, Register::NOREG, cycles_remaining, mem),
                instructions::LDA_ZPX =>
                    cycles_remaining = self.load_zero_page(Register::A, Register::X, cycles_remaining, mem),
                instructions::LDA_ABS =>
                    cycles_remaining = self.load_absolute(Register::A, cycles_remaining, mem),
                instructions::LDA_ABX =>
                    cycles_remaining = self.load_absolute_reg(Register::A, Register::X, cycles_remaining, mem),
                instructions::LDA_ABY =>
                    cycles_remaining = self.load_absolute_reg(Register::A, Register::Y, cycles_remaining, mem),

                // LDX
                instructions::LDX_IMM =>
                    cycles_remaining = self.load_immediate(Register::X, cycles_remaining, mem),
                instructions::LDX_ZP =>
                    cycles_remaining = self.load_zero_page(Register::X, Register::NOREG, cycles_remaining, mem),
                instructions::LDX_ZPY =>
                    cycles_remaining = self.load_zero_page(Register::X, Register::Y, cycles_remaining, mem),
                instructions::LDX_ABS =>
                    cycles_remaining = self.load_absolute(Register::X, cycles_remaining, mem),
                instructions::LDX_ABY =>
                    cycles_remaining = self.load_absolute_reg(Register::X, Register::Y, cycles_remaining, mem),

                // LDY
                instructions::LDY_IMM =>
                    cycles_remaining = self.load_immediate(Register::Y, cycles_remaining, mem),
                instructions::LDY_ZP =>
                    cycles_remaining = self.load_zero_page(Register::Y, Register::NOREG, cycles_remaining, mem),
                instructions::LDY_ZPX =>
                    cycles_remaining = self.load_zero_page(Register::Y, Register::X, cycles_remaining, mem),
                instructions::LDY_ABS =>
                    cycles_remaining = self.load_absolute(Register::Y, cycles_remaining, mem),
                instructions::LDY_ABX =>
                    cycles_remaining = self.load_absolute_reg(Register::Y, Register::X, cycles_remaining, mem),

                _ => todo!()
            }
        }

        cycles - cycles_remaining
    }

    fn load_immediate(&mut self, target_register: Register, cycles: i32, mem: &MEM) -> i32 {
        let (val, nc) = self.fetch_byte(cycles, mem);

        self.set_register(target_register, val);
        self.set_load_flags(val);

        nc
    }

    fn set_load_flags(&mut self, val: Byte) {
        self.sf.set(StatusFlags::ZERO, val == 0);
        self.sf.set(StatusFlags::NEGATIVE, (1 << 7 & val) > 0);
    }

    fn load_zero_page(&mut self, target_register: Register, add_register: Register, cycles: i32, mem: &MEM) -> i32 {
        let (mut addr, mut nc) = self.fetch_byte(cycles, mem);

        addr += match add_register {
            Register::X => self.x,
            Register::Y => self.y,
            Register::NOREG => {nc += 1; 0},
            _ => {println!("Incorrect Instruction, wtf"); exit(0)}
        };

        nc -= 1;

        let (val, nc2) = self.read_byte_from_address(nc, addr as Word, mem);


        self.set_register(target_register, val);
        self.set_load_flags(val);

        nc2
    }

    fn load_absolute(&mut self, target_register: Register, cycles: i32, mem: &MEM) -> i32 {
        let (addr1, nc) = self.fetch_byte(cycles, mem);
        let (addr2, nc2) = self.fetch_byte(nc, mem);

        let addr = (addr1 as Word) | ((addr2 as Word) << 8);

        let (val, nc3) = self.read_byte_from_address(nc2, addr, mem);


        self.set_register(target_register, val);
        self.set_load_flags(val);

        nc3
    }

    fn load_absolute_reg(&mut self, target_register: Register, add_register: Register, cycles: i32, mem: &MEM) -> i32 {
        let (addr1, nc) = self.fetch_byte(cycles, mem);
        let (addr2, nc2) = self.fetch_byte(nc, mem);

        let mut addr = (addr1 as Word) | ((addr2 as Word) << 8);

        addr += match add_register {
            Register::X => self.x,
            Register::Y => self.y,
            _ => { println!("Addressing mode unsupported"); exit(1)}
        } as Word;

        let (val, nc3) = self.read_byte_from_address(nc2, addr, mem);


        self.set_register(target_register, val);
        self.set_load_flags(val);

        nc3
    }

    fn set_register(&mut self, target_register: Register, value: Byte) {
        match target_register {
            Register::A => self.a = value,
            Register::X => self.x = value,
            Register::Y => self.x = value,
            _ => {
                println!("Invalid call - Load only supports registers A, X, Y");
                exit(1)
            }
        }
    }

    fn fetch_byte(&mut self, cycles: i32, mem: &MEM) -> (Byte, i32) {
        let val = mem[self.pc];
        self.pc += 1;

        (val, cycles - 1)
    }

    fn read_byte_from_address(&mut self, cycles: i32, addr: Word, mem: &MEM) -> (Byte, i32) {
        (mem[addr], cycles - (if addr > 0x0100 {2} else {1}))
    }
}