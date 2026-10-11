use super::instructions;

pub type Byte = u8;
pub type Word = u16;

pub type Bit = bool;
pub const ZERO: Bit = false;
pub const ONE: Bit = true;

pub const MAX_MEMORY: usize = 1024 * 64; // 64 Kb

pub trait Bus {
    // Memory
    fn read (&self, address: Word) -> Byte;
    fn write (&mut self, address: Word, data: Byte);

    // Clock
    fn tick(&mut self, cycles: usize) -> usize;
    fn clock(&self) -> usize;
}

pub struct MockRAM {
    data: [Byte; MAX_MEMORY],
    clock: usize
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatusFlags(pub Byte);

#[derive(Copy, Clone)]
pub struct CPU {
    // Registers
    pub(crate) pc: Word, // Program counter
    pub(crate) sp: Byte, // Stack pointer
    pub(crate) a:  Byte, // Accumulator
    pub(crate) x:  Byte, // Index Register X
    pub(crate) y:  Byte, // Index Register Y
    pub(crate) sf: StatusFlags, // Status Flags
}

pub enum Register {
    PC, SP, A, X, Y
}

pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndexedIndirect,
    IndirectIndexed
}

impl MockRAM {
    pub fn init(&mut self) {
        self.data.fill(0)
    }
}

impl Default for MockRAM {
    fn default() -> Self {
        MockRAM {
            data: [0; MAX_MEMORY],
            clock: 0
        }
    }
}

impl Bus for MockRAM {
    fn read(&self, address: Word) -> Byte {
        self.data[address as usize]
    }

    fn write(&mut self, address: Word, data: Byte) {
        self.data[address as usize] = data;
    }

    fn tick(&mut self, cycles: usize) -> usize {
        self.clock += cycles;

        self.clock
    }

    fn clock(&self) -> usize {
        self.clock
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
            sf: StatusFlags(0),
        }
    }
}
impl CPU {
    pub fn reset(&mut self, mem: &mut MockRAM) {
        self.reset_with_address(0xFFFC, mem)
    }

    fn reset_with_address(&mut self, reset_vector: Word, mem: &mut MockRAM) {
        self.pc = reset_vector;
        self.sp = 0xFF;
        self.x = 0;
        self.y = 0;
        self.a = 0;
        self.sf.zero();
        mem.init();
    }

    pub fn execute<B: Bus>(&mut self, cycles: usize, bus: &mut B) -> usize {
        let mut cycles_remaining = cycles;

        while cycles_remaining > 0 {
            let clock_start = bus.clock();
            let instruction = self.fetch_byte(bus);
            bus.tick(1);

            match instruction {
                // LDA
                instructions::LDA_IMM  => self.lda(bus, AddressingMode::Immediate),
                instructions::LDA_ZP => self.lda(bus, AddressingMode::ZeroPage),
                instructions::LDA_ZPX => self.lda(bus, AddressingMode::ZeroPageX),
                instructions::LDA_ABS => self.lda(bus, AddressingMode::Absolute),
                instructions::LDA_ABX => self.lda(bus, AddressingMode::AbsoluteX),
                instructions::LDA_ABY => self.lda(bus, AddressingMode::AbsoluteY),
                instructions::LDA_IDX => self.lda(bus, AddressingMode::IndexedIndirect),
                instructions::LDA_IDY => self.lda(bus, AddressingMode::IndirectIndexed),


                // LDX
                instructions::LDX_IMM => self.ldx(bus, AddressingMode::Immediate),
                instructions::LDX_ZP => self.ldx(bus, AddressingMode::ZeroPage),
                instructions::LDX_ZPY => self.ldx(bus, AddressingMode::ZeroPageY),
                instructions::LDX_ABS => self.ldx(bus, AddressingMode::Absolute),
                instructions::LDX_ABY => self.ldx(bus, AddressingMode::AbsoluteY),

                // LDY
                instructions::LDY_IMM => self.ldy(bus, AddressingMode::Immediate),
                instructions::LDY_ZP => self.ldy(bus, AddressingMode::ZeroPage),
                instructions::LDY_ZPX => self.ldy(bus, AddressingMode::ZeroPageX),
                instructions::LDY_ABS => self.ldy(bus, AddressingMode::Absolute),
                instructions::LDY_ABX => self.ldy(bus, AddressingMode::AbsoluteX),

                _ => panic!("Instruction not implemented: {:#04X} at PC={:#06X}", instruction, self.pc)
            }

            cycles_remaining = cycles_remaining.wrapping_sub(bus.clock() - clock_start);
        }

        cycles - cycles_remaining
    }

    fn lda<B: Bus>(&mut self, bus : &mut B, addressing_mode: AddressingMode) {
        let addr = self.resolve_address(bus, addressing_mode);
        self.a = bus.read(addr);
        bus.tick(1);
        self.set_nz(self.a)
    }

    fn ldx<B: Bus>(&mut self, bus : &mut B, addressing_mode: AddressingMode) {
        let addr = self.resolve_address(bus, addressing_mode);
        self.x = bus.read(addr);
        bus.tick(1);
        self.set_nz(self.x)
    }

    fn ldy<B: Bus>(&mut self, bus : &mut B, addressing_mode: AddressingMode) {
        let addr = self.resolve_address(bus, addressing_mode);
        self.y = bus.read(addr);
        bus.tick(1);
        self.set_nz(self.y)
    }

    fn set_nz(&mut self, val: Byte) {
        self.sf.set(StatusFlags::ZERO, val == 0);
        self.sf.set(StatusFlags::NEGATIVE, (1 << 7 & val) > 0);
    }

    fn resolve_address<B: Bus>(&mut self, bus: &mut B, mode: AddressingMode) -> Word {
        match mode {
            AddressingMode::Immediate => {
                let addr = self.pc;
                self.pc = self.pc.wrapping_add(1);

                addr
            },
            AddressingMode::ZeroPage => {
                bus.tick(1);
                self.fetch_byte(bus) as Word
            }
            AddressingMode::ZeroPageX => {
                bus.tick(2);
                self.fetch_byte(bus).wrapping_add(self.x) as Word
            }
            AddressingMode::ZeroPageY => {
                bus.tick(2);
                self.fetch_byte(bus).wrapping_add(self.y) as Word
            }
            AddressingMode::Absolute => {
                bus.tick(2);
                (self.fetch_byte(bus) as Word) | ((self.fetch_byte(bus) as Word) << 8)
            }
            AddressingMode::AbsoluteX => {
                let original_addr = (self.fetch_byte(bus) as Word) | ((self.fetch_byte(bus) as Word) << 8); 
                let addr = original_addr.wrapping_add(self.x as Word);
                
                if original_addr & 0xFF00 != addr & 0xFF00 {
                    bus.tick(3);
                } else {
                    bus.tick(2);
                }
                
                addr
            }
            AddressingMode::AbsoluteY => {
                let original_addr = (self.fetch_byte(bus) as Word) | ((self.fetch_byte(bus) as Word) << 8);
                let addr = original_addr.wrapping_add(self.y as Word);

                // Page crossing
                if original_addr & 0xFF00 != addr & 0xFF00 {
                    bus.tick(3);
                } else {
                    bus.tick(2);
                }

                addr
            }
            AddressingMode::IndexedIndirect => {
                let starting_addr = (self.fetch_byte(bus)).wrapping_add(self.x);
                let lsb = bus.read(starting_addr as Word);
                let msb = bus.read(starting_addr.wrapping_add(1) as Word);

                bus.tick(4);

                (lsb as Word) | ((msb as Word) << 8)
            }
            AddressingMode::IndirectIndexed => {
                let starting_addr = self.fetch_byte(bus);
                let lsb = bus.read(starting_addr as Word);
                let msb = bus.read(starting_addr.wrapping_add(1) as Word);
                let original_addr = (lsb as Word) | ((msb as Word) << 8);
                let addr = original_addr.wrapping_add(self.y as Word);

                // Page crossing
                if original_addr & 0xFF00 != addr & 0xFF00 {
                    bus.tick(4);
                } else {
                    bus.tick(3);
                }

                addr


            }
        }
    }

    fn fetch_byte<B: Bus>(&mut self, mem: &mut B) -> Byte {
        let val = mem.read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        val
    }
}