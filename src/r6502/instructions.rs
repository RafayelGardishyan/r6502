// LDA
pub const LDA_IMM: u8 = 0xA9;
pub const LDA_ZP: u8  = 0xA5;
pub const LDA_ZPX: u8 = 0xB5;
pub const LDA_ABS: u8 = 0xAD;
pub const LDA_ABX: u8 = 0xBD;
pub const LDA_ABY: u8 = 0xB9;
pub const LDA_IDX: u8 = 0xA1;
pub const LDA_IDY: u8 = 0xB1;

// LDX
pub const LDX_IMM: u8 = 0xA2;
pub const LDX_ZP: u8 = 0xA6;
pub const LDX_ZPY: u8 = 0xB6;
pub const LDX_ABS: u8 = 0xAE;
pub const LDX_ABY: u8 = 0xBE;

// LDY
pub const LDY_IMM: u8 = 0xA0;
pub const LDY_ZP: u8 = 0xA4;
pub const LDY_ZPX: u8 = 0xB4;
pub const LDY_ABS: u8 = 0xAC;
pub const LDY_ABX: u8 = 0xBC;

// STA
pub const STA_ZP: u8 = 0x85;
pub const STA_ZPX: u8 = 0x95;
pub const STA_ABS: u8 = 0x8D;
pub const STA_ABX: u8 = 0x9D;
pub const STA_ABY: u8 = 0x99;
pub const STA_IDX: u8 = 0x81;
pub const STA_IDY: u8 = 0x91;

// STX
pub const STX_ZP: u8 = 0x86;
pub const STX_ZPY: u8 = 0x96;
pub const STX_ABS: u8 = 0x8E;

// STY
pub const STY_ZP: u8 = 0x84;
pub const STY_ZPX: u8 = 0x94;
pub const STY_ABS: u8 = 0x8C;

// Register Transfers (TAX TAY TXA TYA TSX TXS)
pub const TAX: u8 = 0xAA;
pub const TAY: u8 = 0xA8;
pub const TXA: u8 = 0x8A;
pub const TYA: u8 = 0x98;
pub const TSX: u8 = 0xBA;
pub const TXS: u8 = 0x9A;

// Stack Operations
pub const PHA: u8 = 0x48;
pub const PHP: u8 = 0x08;
pub const PLA: u8 = 0x68;
pub const PLP: u8 = 0x28;

// AND
pub const AND_IMM: u8 = 0x29;
pub const AND_ZP: u8 = 0x25;
pub const AND_ZPX: u8 = 0x35;
pub const AND_ABS: u8 = 0x2D;
pub const AND_ABX: u8 = 0x3D;
pub const AND_ABY: u8 = 0x39;
pub const AND_IDX: u8 = 0x21;
pub const AND_IDY: u8 = 0x31;

// EOR
pub const EOR_IMM: u8 = 0x49;
pub const EOR_ZP: u8 = 0x45;
pub const EOR_ZPX: u8 = 0x55;
pub const EOR_ABS: u8 = 0x4D;
pub const EOR_ABX: u8 = 0x5D;
pub const EOR_ABY: u8 = 0x59;
pub const EOR_IDX: u8 = 0x41;
pub const EOR_IDY: u8 = 0x51;

// OR
pub const ORA_IMM: u8 = 0x09;
pub const ORA_ZP: u8 = 0x05;
pub const ORA_ZPX: u8 = 0x15;
pub const ORA_ABS: u8 = 0x0D;
pub const ORA_ABX: u8 = 0x1D;
pub const ORA_ABY: u8 = 0x19;
pub const ORA_IDX: u8 = 0x01;
pub const ORA_IDY: u8 = 0x11;

// BIT
pub const BIT_ZP: u8 = 0x24;
pub const BIT_ABS: u8 = 0x2C;
