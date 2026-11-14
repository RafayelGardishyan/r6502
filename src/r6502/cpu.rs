use std::cmp::PartialEq;
use crate::r6502::instructions::{Instruction, READ_ONLY};
use std::fmt;
use std::fmt::Formatter;

pub type Byte = u8;
pub type Word = u16;

pub type Bit = bool;
pub const ZERO: Bit = false;
pub const ONE: Bit = true;

pub const MAX_MEMORY: usize = 1024 * 64; // 64 Kb
pub type EmulatorResult<T> = std::result::Result<T, &'static str>;

pub trait Bus {
    // Memory
    fn read(&self, address: Word) -> EmulatorResult<Byte>;
    fn write(&mut self, address: Word, data: Byte) -> EmulatorResult<()>;

    // Clock
    fn tick(&mut self, cycles: usize) -> EmulatorResult<usize>;
    fn clock(&self) -> EmulatorResult<usize>;
}

pub struct MockRAM {
    pub(crate) data: [Byte; MAX_MEMORY],
    clock: usize,
}

#[derive(Clone, Copy, PartialEq)]
pub struct StatusFlags(pub Byte);

#[derive(Copy, Clone)]
pub struct CPU {
    // Registers
    pub(crate) pc: Word,        // Program counter
    pub(crate) sp: Byte,        // Stack pointer
    pub(crate) a: Byte,         // Accumulator
    pub(crate) x: Byte,         // Index Register X
    pub(crate) y: Byte,         // Index Register Y
    pub(crate) sf: StatusFlags, // Status Flags

    // Cycle stepped
    cycle: usize,                            // Current instruction cycle
    exec_cycle: usize,                       // Current execution cycle
    current_instruction: Instruction,        // What instruction are we executing
    current_addressing_mode: AddressingMode, // What addressing mode does the instruction have

    // Hidden state
    current_address: Word, // Current address assembled from memory operations
    pointer: Byte,         // Temporary address register
    address_ready: bool,   // Flag to indicate address is done to read
}

pub enum Register {
    PC,
    SP,
    A,
    X,
    Y,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndexedIndirect,
    IndirectIndexed,
    Accumulator,
    Implied
}

pub enum Operation {
    And,
    XOr,
    Or,
    Bit,
    AddCarry,
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
            clock: 0,
        }
    }
}

impl Bus for MockRAM {
    fn read(&self, address: Word) -> EmulatorResult<Byte> {
        Ok(self.data[address as usize])
    }

    fn write(&mut self, address: Word, data: Byte) -> EmulatorResult<()> {
        self.data[address as usize] = data;

        Ok(())
    }

    fn tick(&mut self, cycles: usize) -> EmulatorResult<usize> {
        self.clock += cycles;

        Ok(self.clock)
    }

    fn clock(&self) -> EmulatorResult<usize> {
        Ok(self.clock)
    }
}

impl StatusFlags {
    pub const CARRY: u8 = 1 << 0;
    pub const ZERO: u8 = 1 << 1;
    pub const INTERRUPT: u8 = 1 << 2;
    pub const DECIMAL: u8 = 1 << 3;
    pub const BREAK: u8 = 1 << 4;
    pub const UNUSED: u8 = 1 << 5;
    pub const OVERFLOW: u8 = 1 << 6;
    pub const NEGATIVE: u8 = 1 << 7;

    pub fn set(&mut self, flag: u8, value: Bit) {
        if value || flag == Self::UNUSED{
            self.0 |= flag;
        } else {
            self.0 &= !flag;
        }
    }

    pub fn get(&self, flag: u8) -> Bit {
        if flag == Self::UNUSED {
            return ONE
        }

        (self.0 & flag) != 0
    }

    pub fn zero(&mut self) {
        self.0 &= 0
    }
}

impl fmt::Debug for StatusFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "StatusFlags(0b{:08b})", self.0)
    }
}

impl Default for CPU {
    fn default() -> Self {
        CPU {
            pc: 0,
            sp: 0xFF,
            a: 0,
            x: 0,
            y: 0,
            sf: StatusFlags(0),
            cycle: 0,
            exec_cycle: 0,
            current_instruction: Instruction::NOP,
            current_addressing_mode: AddressingMode::Immediate,
            current_address: 0,
            pointer: 0,
            address_ready: false,
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

        // Reset state
        self.cycle = 0;
        self.exec_cycle = 0;
        self.current_instruction = Instruction::NOP;
        self.current_addressing_mode = AddressingMode::Immediate;
        self.current_address = 0;
        self.pointer = 0;
        self.address_ready = false;

        mem.init();
    }

    pub fn execute<B: Bus>(&mut self, cycles: usize, bus: &mut B) -> EmulatorResult<usize> {
        let mut cycles_remaining = cycles;

        while cycles_remaining > 0 {
            self.execute_cycle(bus).expect("Error while executing");

            cycles_remaining -= 1;
        }

        Ok(cycles - cycles_remaining)
    }

    pub fn execute_cycle<B: Bus>(&mut self, bus: &mut B) -> EmulatorResult<()> {
        match (self.cycle, self.address_ready) {
            (0, false) => {
                self.fetch_and_decode(bus)
                    .expect("CPU Error: failed to decode");
                self.cycle += 1;
                self.exec_cycle = 0;
            }
            (_, false) => {
                self.resolve_address(bus)
                    .expect("CPU Error: address resolution failed");

                self.cycle += 1;
            }
            (_, true) => {
                self.execute_operation(bus)
                    .expect("CPU Error: execution failed");
                self.exec_cycle += 1;
            }
        }

        Ok(())
    }

    pub fn set_register(&mut self, register: Register, value: Byte) -> EmulatorResult<()> {
        match register {
            Register::SP => {
                self.sp = value;
                return Ok(())
            }
            Register::A => {
                self.a = value;

            }
            Register::X => {
                self.x = value;
            }
            Register::Y => {
                self.y = value;
            }
            _ => return Err("Cannot set PC from here")
        }
        self.set_nz(value);

        Ok(())
    }

    pub fn execute_operation<B: Bus>(&mut self, bus: &mut B) -> EmulatorResult<()> {
        match self.current_instruction {
            Instruction::LDA => {
                assert_eq!(self.exec_cycle, 0);

                self.a = bus.read(self.current_address)?;
                self.set_nz(self.a);

                self.finalise_execution();
            }
            Instruction::LDX => {
                assert_eq!(self.exec_cycle, 0);

                self.x = bus.read(self.current_address)?;
                self.set_nz(self.x);

                self.finalise_execution();
            }
            Instruction::LDY => {
                assert_eq!(self.exec_cycle, 0);

                self.y = bus.read(self.current_address)?;
                self.set_nz(self.y);

                self.finalise_execution();
            }
            Instruction::STA => {
                assert_eq!(self.exec_cycle, 0);

                bus.write(self.current_address, self.a)?;

                self.finalise_execution();
            }
            Instruction::STX => {
                assert_eq!(self.exec_cycle, 0);

                bus.write(self.current_address, self.x)?;

                self.finalise_execution();
            }
            Instruction::STY => {
                assert_eq!(self.exec_cycle, 0);

                bus.write(self.current_address, self.y)?;

                self.finalise_execution();
            }
            Instruction::TAX => {
                assert_eq!(self.exec_cycle, 0);

                self.set_register(Register::X, self.a)?;
                self.finalise_execution();
            }
            Instruction::TAY => {
                assert_eq!(self.exec_cycle, 0);

                self.set_register(Register::Y, self.a)?;
                self.finalise_execution();
            }
            Instruction::TXA => {
                assert_eq!(self.exec_cycle, 0);

                self.set_register(Register::A, self.x)?;
                self.finalise_execution();
            }
            Instruction::TYA => {
                assert_eq!(self.exec_cycle, 0);

                self.set_register(Register::A, self.y)?;
                self.finalise_execution();
            }
            Instruction::TSX => {
                assert_eq!(self.exec_cycle, 0);

                self.set_register(Register::X, self.sp)?;
                self.finalise_execution();
            }
            Instruction::TXS => {
                assert_eq!(self.exec_cycle, 0);

                self.set_register(Register::SP, self.x)?;
                self.finalise_execution();
            }
            Instruction::PHA => {
                match self.exec_cycle {
                    0 => {
                        bus.read(self.pc)?; // Throw read
                    }
                    1 => {
                        bus.write(self.sp() as Word, self.a)?;
                        self.sp = self.sp.wrapping_sub(1);
                        self.finalise_execution();
                    }
                    _ => return Err("Wrong cycle for instruction")
                }
            }
            Instruction::PHP => {
                match self.exec_cycle {
                    0 => {
                        bus.read(self.pc)?; // Throw read
                    }
                    1 => {
                        bus.write(self.sp(), read_sf(self.sf))?;
                        self.sp = self.sp.wrapping_sub(1);
                        self.finalise_execution();
                    }
                    _ => return Err("Wrong cycle for instruction")
                }
            }
            Instruction::PLA => {
                match self.exec_cycle {
                    0 => {
                        bus.read(self.pc)?; // Throw read
                    }
                    1 => {
                        self.sp = self.sp.wrapping_add(1);
                    }
                    2 => {
                        self.set_register(Register::A, bus.read(self.sp())?)?;
                        self.finalise_execution();
                    }
                    _ => return Err("Wrong cycle for instruction")
                }
            }
            Instruction::PLP => {
                match self.exec_cycle {
                    0 => {
                        bus.read(self.pc)?; // Throw read
                    }
                    1 => {
                        self.sp = self.sp.wrapping_add(1);
                    }
                    2 => {
                        let read = bus.read(self.sp())?;
                        self.sf.0 = read & 0b11001111 | 0b00100000; // Ignore bits 4, 5 and set 5 to 1
                        self.finalise_execution();
                    }
                    _ => return Err("Wrong cycle for instruction")
                }
            }
            Instruction::AND => {
                assert_eq!(self.exec_cycle, 0);

                let value = bus.read(self.current_address)?;
                self.set_register(Register::A, self.a & value)?;
                self.finalise_execution();
            }
            Instruction::EOR => {
                assert_eq!(self.exec_cycle, 0);

                let value = bus.read(self.current_address)?;
                self.set_register(Register::A, self.a ^ value)?;
                self.finalise_execution();
            }
            Instruction::ORA => {
                assert_eq!(self.exec_cycle, 0);

                let value = bus.read(self.current_address)?;
                self.set_register(Register::A, self.a | value)?;
                self.finalise_execution();
            }
            Instruction::BIT => {
                assert_eq!(self.exec_cycle, 0);

                let value = bus.read(self.current_address)?;

                self.sf.set(StatusFlags::ZERO, self.a & value == 0);
                self.sf.set(StatusFlags::OVERFLOW, value & 0b01000000 > 0);
                self.sf.set(StatusFlags::NEGATIVE, value & 0b10000000 > 0);

                self.finalise_execution();
            }
            Instruction::ADC => {
                assert_eq!(self.exec_cycle, 0);
                let val = bus.read(self.current_address)?;
                let carry_in = if self.sf.get(StatusFlags::CARRY) { 1 } else { 0 };
                let old_a = self.a;

                if self.sf.get(StatusFlags::DECIMAL) {
                    let mut al = (old_a & 0x0F) + (val & 0x0F) + carry_in;
                    if al > 0x09 {
                        al = al.wrapping_add(0x06);
                    }

                    let mut ah = (old_a >> 4) + (val >> 4) + (if al > 0x0F { 1 } else { 0 });

                    let bin_sum = old_a.wrapping_add(val).wrapping_add(carry_in);
                    let intermediate = (ah << 4) | (al & 0x0F);

                    self.sf.set(StatusFlags::ZERO, bin_sum == 0);
                    self.sf.set(StatusFlags::NEGATIVE, (ah & 0x08) != 0);
                    self.sf.set(
                        StatusFlags::OVERFLOW,
                        ((old_a ^ intermediate) & (val ^ intermediate) & 0x80) != 0,
                    );

                    if ah > 0x09 {
                        ah = ah.wrapping_add(0x06);
                    }

                    self.sf.set(StatusFlags::CARRY, ah > 0x0F);

                    self.a = ((ah << 4) | (al & 0x0F)) & 0xFF;
                } else {
                    let (nv1, of1) = old_a.overflowing_add(val);
                    let (nv2, of2) = nv1.overflowing_add(carry_in);

                    self.sf.set(StatusFlags::CARRY, of1 || of2);
                    self.sf.set(
                        StatusFlags::OVERFLOW,
                        ((old_a ^ nv2) & (val ^ nv2) & 0x80) != 0,
                    );

                    self.set_register(Register::A, nv2)?;
                }

                self.finalise_execution();
            }
            Instruction::NOP => {}
        }

        Ok(())
    }

    fn is_set_bit(val: Byte, bit: usize) -> bool {
        val & (1 << bit) != 0
    }

    fn set_nz(&mut self, val: Byte) {
        self.sf.set(StatusFlags::ZERO, val == 0x00);
        self.sf.set(StatusFlags::NEGATIVE, Self::is_set_bit(val, 7));
    }

    fn resolve_address<B: Bus>(&mut self, bus: &mut B) -> EmulatorResult<()> {
        match self.current_addressing_mode {
            AddressingMode::Immediate => {
                return Err("Address resolution error: Immediates handled in decode");
            }
            AddressingMode::ZeroPage => match self.cycle {
                1 => {
                    self.current_address = bus.read(self.pc).expect("Memory read error") as Word;
                    self.increment_pc();
                    self.address_ready = true;
                }
                _ => return Err("Address resolution error: invalid cycle for addressing mode"),
            },
            AddressingMode::ZeroPageX => match self.cycle {
                1 => {
                    self.pointer = bus.read(self.pc).expect("Memory read error");
                    self.increment_pc();
                }
                2 => {
                    self.current_address = self.pointer.wrapping_add(self.x) as Word;
                    self.address_ready = true;
                }
                _ => return Err("Address resolution error: invalid cycle for addressing mode"),
            },
            AddressingMode::ZeroPageY => match self.cycle {
                1 => {
                    self.pointer = bus.read(self.pc).expect("Memory read error");
                    self.increment_pc()
                }
                2 => {
                    self.current_address = self.pointer.wrapping_add(self.y) as Word;
                    self.address_ready = true;
                }
                _ => return Err("Address resolution error: invalid cycle for addressing mode"),
            },
            AddressingMode::Absolute => match self.cycle {
                1 => {
                    self.current_address = bus.read(self.pc).expect("Memory read error") as Word;
                    self.increment_pc();
                }
                2 => {
                    self.current_address |=
                        (bus.read(self.pc).expect("Memory read error") as Word) << 8;
                    self.increment_pc();
                    self.address_ready = true;
                }
                _ => return Err("Address resolution error: invalid cycle for addressing mode"),
            },
            AddressingMode::AbsoluteX => match self.cycle {
                1 => {
                    self.current_address = bus.read(self.pc).expect("Memory read error") as Word;
                    self.increment_pc();
                }
                2 => {
                    self.current_address |=
                        (bus.read(self.pc).expect("Memory read error") as Word) << 8;
                    self.increment_pc();

                    let new_address = self.current_address.wrapping_add(self.x as Word);

                    self.address_ready = self.read_only() && (self.current_address & 0xFF00 == new_address & 0xFF00);

                    self.current_address = new_address;
                }
                3 => {
                    self.address_ready = true;
                }
                _ => return Err("Address resolution error: invalid cycle for addressing mode"),
            },
            AddressingMode::AbsoluteY => match self.cycle {
                1 => {
                    self.current_address = bus.read(self.pc).expect("Memory read error") as Word;
                    self.increment_pc();
                }
                2 => {
                    self.current_address |=
                        (bus.read(self.pc).expect("Memory read error") as Word) << 8;
                    self.increment_pc();

                    let new_address = self.current_address.wrapping_add(self.y as Word);

                    self.address_ready = self.read_only() && (self.current_address & 0xFF00 == new_address & 0xFF00);

                    self.current_address = new_address;
                }
                3 => {
                    self.address_ready = true;
                }
                _ => return Err("Address resolution error: invalid cycle for addressing mode"),
            },
            AddressingMode::IndexedIndirect => match self.cycle {
                1 => {
                    self.pointer = bus.read(self.pc).expect("Memory read error");
                    self.increment_pc();
                }
                2 => {
                    self.pointer = self.pointer.wrapping_add(self.x);
                }
                3 => {
                    self.current_address =
                        bus.read(self.pointer as Word).expect("Memory read error") as Word;
                }
                4 => {
                    self.current_address |=
                        (bus.read((self.pointer.wrapping_add(1)) as Word).expect("Memory read error") as Word) << 8;
                    self.address_ready = true;
                }
                _ => return Err("Address resolution error: invalid cycle for addressing mode"),
            },
            AddressingMode::IndirectIndexed => match self.cycle {
                1 => {
                    self.pointer = bus.read(self.pc).expect("Memory read error");
                    self.increment_pc();
                }
                2 => {
                    self.current_address =
                        bus.read(self.pointer as Word).expect("Memory read error") as Word;
                }
                3 => {
                    self.current_address |=
                        (bus.read((self.pointer.wrapping_add(1)) as Word).expect("Memory read error") as Word) << 8;

                    let new_address = self.current_address.wrapping_add(self.y as Word);

                    self.address_ready = self.read_only() && (self.current_address & 0xFF00 == new_address & 0xFF00);

                    self.current_address = new_address;
                }
                4 => {
                    self.address_ready = true;
                }
                _ => return Err("Address resolution error: invalid cycle for addressing mode"),
            },
            AddressingMode::Implied => return Err("Implied address no resolution needed"),
            _ => return Err("Not implemented yet")
        }

        Ok(())
    }

    fn increment_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1)
    }

    fn fetch_and_decode<B: Bus>(&mut self, bus: &mut B) -> EmulatorResult<()> {
        let opcode = bus.read(self.pc)?;

        let cc = opcode & 0x03;
        let bbb = (opcode >> 2) & 0x07;
        let aaa = (opcode >> 5) & 0x07;

        let implied_instruction = match opcode {
            0xAA => Some(Instruction::TAX),
            0xA8 => Some(Instruction::TAY),
            0x8A => Some(Instruction::TXA),
            0x98 => Some(Instruction::TYA),
            0xBA => Some(Instruction::TSX),
            0x9A => Some(Instruction::TXS),
            0x48 => Some(Instruction::PHA),
            0x08 => Some(Instruction::PHP),
            0x68 => Some(Instruction::PLA),
            0x28 => Some(Instruction::PLP),
            0xEA => Some(Instruction::NOP),
            _ => None,
        };

        if let Some(instr) = implied_instruction {
            self.current_instruction = instr;
            self.current_addressing_mode = AddressingMode::Implied;
            self.increment_pc();
            self.address_ready = true; // Execute directly without any access
            return Ok(());
        }

        self.current_addressing_mode = match cc {
            0b01 => match bbb {
                0b000 => AddressingMode::IndexedIndirect,
                0b001 => AddressingMode::ZeroPage,
                0b010 => AddressingMode::Immediate,
                0b011 => AddressingMode::Absolute,
                0b100 => AddressingMode::IndirectIndexed,
                0b101 => AddressingMode::ZeroPageX,
                0b110 => AddressingMode::AbsoluteY,
                0b111 => AddressingMode::AbsoluteX,
                _ => {
                    return Err(
                        "Encountered malformed instruction while decoding: wrong addressing mode",
                    );
                }
            },
            0b10 => match bbb {
                0b000 => AddressingMode::Immediate,
                0b001 => AddressingMode::ZeroPage,
                0b010 => AddressingMode::Accumulator,
                0b011 => AddressingMode::Absolute,
                0b101 => AddressingMode::ZeroPageX,
                0b111 => AddressingMode::AbsoluteX,
                _ => {
                    return Err(
                        "Encountered malformed instruction while decoding: wrong addressing mode",
                    );
                }
            },
            0b00 => match bbb {
                0b000 => AddressingMode::Immediate,
                0b001 => AddressingMode::ZeroPage,
                0b011 => AddressingMode::Absolute,
                0b101 => AddressingMode::ZeroPageX,
                0b111 => AddressingMode::AbsoluteX,
                _ => {
                    return Err(
                        "Encountered malformed instruction while decoding: wrong addressing mode",
                    );
                }
            },
            _ => {
                return Err(
                    "Encountered malformed instruction while decoding: wrong instruction group",
                );
            }
        };

        self.current_instruction = match cc {
            0b01 => match aaa {
                0b000 => Instruction::ORA,
                0b001 => Instruction::AND,
                0b010 => Instruction::EOR,
                0b011 => Instruction::ADC,
                0b100 => Instruction::STA,
                0b101 => Instruction::LDA,
                // todo implement
                // 0b110 => Instruction::CMP,
                // 0b111 => Instruction::SBC,
                _ => {
                    return Err(
                        "Encountered malformed instruction while decoding: wrong operation",
                    );
                }
            },
            0b10 => match aaa {
                // todo implement
                // 0b000 => Instruction::ASL,
                // 0b001 => Instruction::ROL,
                // 0b010 => Instruction::LSR,
                // 0b011 => Instruction::ROR,
                0b100 => Instruction::STX,
                0b101 => Instruction::LDX,
                // todo implement
                // 0b110 => Instruction::DEC,
                // 0b111 => Instruction::INC,
                _ => {
                    return Err(
                        "Encountered malformed instruction while decoding: wrong operation",
                    );
                }
            },
            0b00 => match aaa {
                0b001 => Instruction::BIT,
                // todo implement
                // 0b010 => Instruction::JMP,
                // 0b011 => Instruction::JMP,
                0b100 => Instruction::STY,
                0b101 => Instruction::LDY,
                // todo implement
                // 0b110 => Instruction::CPY,
                // 0b111 => Instruction::CPX,
                _ => {
                    return Err(
                        "Encountered malformed instruction while decoding: wrong operation",
                    );
                }
            },
            _ => {
                return Err(
                    "Encountered malformed instruction while decoding: wrong instruction group",
                );
            }
        };

        // Exceptions
        match (self.current_instruction, self.current_addressing_mode) {
            (Instruction::LDX, AddressingMode::ZeroPageX) => {
                self.current_addressing_mode = AddressingMode::ZeroPageY
            }
            (Instruction::STX, AddressingMode::ZeroPageX) => {
                self.current_addressing_mode = AddressingMode::ZeroPageY
            }
            (Instruction::LDX, AddressingMode::AbsoluteX) => {
                self.current_addressing_mode = AddressingMode::AbsoluteY
            }
            _ => {}
        }

        // Increment PC
        self.increment_pc();

        // We don't waste cycles on immediate address resolution - we already have it
        if self.current_addressing_mode == AddressingMode::Immediate {
            self.current_address = self.pc;
            self.increment_pc();

            self.address_ready = true;
        }

        Ok(())
    }

    fn finalise_execution(&mut self) {
        self.cycle = 0;
        self.address_ready = false;
    }

    fn read_only(&self) -> bool {
        READ_ONLY.contains(&self.current_instruction)
    }

    fn sp(&self) -> Word {
        0x0100 | self.sp as Word
    }
}

fn read_sf(status_flags: StatusFlags) -> Byte {
    status_flags.0 | 0b00110000
}
