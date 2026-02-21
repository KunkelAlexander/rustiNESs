use crate::bus::Bus;

/*
	olc6502 - An emulation of the 6502/2A03 processor
	"Thanks Dad for believing computers were gonna be a big deal..." - javidx9

	License (OLC-3)
	~~~~~~~~~~~~~~~

	Copyright 2018-2019 OneLoneCoder.com

	Redistribution and use in source and binary forms, with or without
	modification, are permitted provided that the following conditions
	are met:

	1. Redistributions or derivations of source code must retain the above
	copyright notice, this list of conditions and the following disclaimer.

	2. Redistributions or derivative works in binary form must reproduce
	the above copyright notice. This list of conditions and the following
	disclaimer must be reproduced in the documentation and/or other
	materials provided with the distribution.

	3. Neither the name of the copyright holder nor the names of its
	contributors may be used to endorse or promote products derived
	from this software without specific prior written permission.

	THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
	"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
	LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
	A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
	HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
	SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
	LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
	DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
	THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
	(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
	OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

	Background
	~~~~~~~~~~
	I love this microprocessor. It was at the heart of two of my favourite
	machines, the BBC Micro, and the Nintendo Entertainment System, as well
	as countless others in that era. I learnt to program on the Model B, and
	I learnt to love games on the NES, so in many ways, this processor is
	why I am the way I am today.

	In February 2019, I decided to undertake a selfish personal project and
	build a NES emulator. Ive always wanted to, and as such I've avoided
	looking at source code for such things. This made making this a real
	personal challenge. I know its been done countless times, and very likely
	in far more clever and accurate ways than mine, but I'm proud of this.

	Datasheet: http://archive.6502.org/datasheets/rockwell_r650x_r651x.pdf

	Files: olc6502.h, olc6502.cpp

	Relevant Video: https://www.youtube.com/watch?v=8XmxKPJDGU0

	Links
	~~~~~
	YouTube:	https://www.youtube.com/javidx9
				https://www.youtube.com/javidx9extra
	Discord:	https://discord.gg/WhwHUMV
	Twitter:	https://www.twitter.com/javidx9
	Twitch:		https://www.twitch.tv/javidx9
	GitHub:		https://www.github.com/onelonecoder
	Patreon:	https://www.patreon.com/javidx9
	Homepage:	https://www.onelonecoder.com

	Author
	~~~~~~
	David Barr, aka javidx9, ©OneLoneCoder 2019

    Translated into Rust by Alexander Kunkel, 2026
    No copy-paste from LLMs or Copilot for this project (except for the opcode lookup table) as I realised I understand what I am doing less when I rely on their code too much :) 

*/


// Note that https://www.nesdev.org/wiki/Instruction_reference refers to the U bit as 1 
// when they write something like the bit order is NV1BDIZC (high to low). 
pub const FLAG6502_C: u8 = 1 << 0; // Carry Bit
pub const FLAG6502_Z: u8 = 1 << 1; // Zero
pub const FLAG6502_I: u8 = 1 << 2; // Disable Interrupts
pub const FLAG6502_D: u8 = 1 << 3; // Decimal Mode (unused in this implementation)
pub const FLAG6502_B: u8 = 1 << 4; // Break
pub const FLAG6502_U: u8 = 1 << 5; // Unused
pub const FLAG6502_V: u8 = 1 << 6; // Overflow
pub const FLAG6502_N: u8 = 1 << 7; // Negative

// Javid9x' code compares function pointers to determine the addressing mode
// I think it would be nicer to store integers in the instruction table and compare these
// The actual lookup is then done using a match instruction
// This enum defines the address modes
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AddressMode {
    IMP,
    IMM,
    ZP0,
    ZPX,
    ZPY,
    ABS,
    ABX,
    ABY,
    IND,
    IZX,
    IZY,
    REL,
}


// Javid9x' code compares function pointers to determine the addressing mode
// I think it would be nicer to store integers in the instruction table and compare these
// The actual lookup is then done using a match instruction
// This enum defines the operations to completely get rid of storing function pointers 
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    // System
    BRK,
    NOP,

    // Loads
    LDA,
    LDX,
    LDY,

    // Stores
    STA,
    STX,
    STY,

    // Register transfers
    TAX,
    TAY,
    TXA,
    TYA,
    TSX,
    TXS,

    // Stack
    PHA,
    PHP,
    PLA,
    PLP,

    // Logical
    AND,
    EOR,
    ORA,
    BIT,

    // Arithmetic
    ADC,
    SBC,
    CMP,
    CPX,
    CPY,

    // Inc / Dec
    INC,
    INX,
    INY,
    DEC,
    DEX,
    DEY,

    // Shifts
    ASL,
    LSR,
    ROL,
    ROR,

    // Jumps / Calls
    JMP,
    JSR,
    RTS,
    RTI,

    // Branches
    BCC,
    BCS,
    BEQ,
    BMI,
    BNE,
    BPL,
    BVC,
    BVS,

    // Flags
    CLC,
    CLD,
    CLI,
    CLV,
    SEC,
    SED,
    SEI,

    // Illegal / placeholder
    XXX,
}



#[derive(Copy, Clone)]
pub struct Instruction {
    pub name: &'static str,
    pub addrmode: AddressMode,
    pub operation: Operation,
    pub cycles: u8,
}


const fn build_lookup() -> [Instruction; 256] {
    // default all opcodes to illegal/unknown
    let xxx = Instruction { name: "???", addrmode: AddressMode::IMP, operation: Operation::XXX, cycles: 2 };
    let mut t = [xxx; 256];

    // Helper macro to make the table readable
    macro_rules! op {
        ($code:expr, $name:expr, $addr:ident, $op:ident, $cy:expr) => {
            t[$code] = Instruction {
                name: $name,
                addrmode: AddressMode::$addr,
                operation: Operation::$op,
                cycles: $cy,
            };
        };
    }

    // ----- Official NMOS 6502 opcodes (base cycles) -----
    op!(0x00, "BRK", IMP, BRK, 7); // Javid uses imm here, but for Harte to pass i need imp
    op!(0x01, "ORA", IZX, ORA, 6);
    op!(0x05, "ORA", ZP0, ORA, 3);
    op!(0x06, "ASL", ZP0, ASL, 5);
    op!(0x08, "PHP", IMP, PHP, 3);
    op!(0x09, "ORA", IMM, ORA, 2);
    op!(0x0A, "ASL", IMP, ASL, 2);
    op!(0x0D, "ORA", ABS, ORA, 4);
    op!(0x0E, "ASL", ABS, ASL, 6);

    op!(0x10, "BPL", REL, BPL, 2);
    op!(0x11, "ORA", IZY, ORA, 5);
    op!(0x15, "ORA", ZPX, ORA, 4);
    op!(0x16, "ASL", ZPX, ASL, 6);
    op!(0x18, "CLC", IMP, CLC, 2);
    op!(0x19, "ORA", ABY, ORA, 4);
    op!(0x1D, "ORA", ABX, ORA, 4);
    op!(0x1E, "ASL", ABX, ASL, 7);

    op!(0x20, "JSR", ABS, JSR, 6);
    op!(0x21, "AND", IZX, AND, 6);
    op!(0x24, "BIT", ZP0, BIT, 3);
    op!(0x25, "AND", ZP0, AND, 3);
    op!(0x26, "ROL", ZP0, ROL, 5);
    op!(0x28, "PLP", IMP, PLP, 4);
    op!(0x29, "AND", IMM, AND, 2);
    op!(0x2A, "ROL", IMP, ROL, 2);
    op!(0x2C, "BIT", ABS, BIT, 4);
    op!(0x2D, "AND", ABS, AND, 4);
    op!(0x2E, "ROL", ABS, ROL, 6);

    op!(0x30, "BMI", REL, BMI, 2);
    op!(0x31, "AND", IZY, AND, 5);
    op!(0x35, "AND", ZPX, AND, 4);
    op!(0x36, "ROL", ZPX, ROL, 6);
    op!(0x38, "SEC", IMP, SEC, 2);
    op!(0x39, "AND", ABY, AND, 4);
    op!(0x3D, "AND", ABX, AND, 4);
    op!(0x3E, "ROL", ABX, ROL, 7);

    op!(0x40, "RTI", IMP, RTI, 6);
    op!(0x41, "EOR", IZX, EOR, 6);
    op!(0x45, "EOR", ZP0, EOR, 3);
    op!(0x46, "LSR", ZP0, LSR, 5);
    op!(0x48, "PHA", IMP, PHA, 3);
    op!(0x49, "EOR", IMM, EOR, 2);
    op!(0x4A, "LSR", IMP, LSR, 2);
    op!(0x4C, "JMP", ABS, JMP, 3);
    op!(0x4D, "EOR", ABS, EOR, 4);
    op!(0x4E, "LSR", ABS, LSR, 6);

    op!(0x50, "BVC", REL, BVC, 2);
    op!(0x51, "EOR", IZY, EOR, 5);
    op!(0x55, "EOR", ZPX, EOR, 4);
    op!(0x56, "LSR", ZPX, LSR, 6);
    op!(0x58, "CLI", IMP, CLI, 2);
    op!(0x59, "EOR", ABY, EOR, 4);
    op!(0x5D, "EOR", ABX, EOR, 4);
    op!(0x5E, "LSR", ABX, LSR, 7);

    op!(0x60, "RTS", IMP, RTS, 6);
    op!(0x61, "ADC", IZX, ADC, 6);
    op!(0x65, "ADC", ZP0, ADC, 3);
    op!(0x66, "ROR", ZP0, ROR, 5);
    op!(0x68, "PLA", IMP, PLA, 4);
    op!(0x69, "ADC", IMM, ADC, 2);
    op!(0x6A, "ROR", IMP, ROR, 2);
    op!(0x6C, "JMP", IND, JMP, 5);
    op!(0x6D, "ADC", ABS, ADC, 4);
    op!(0x6E, "ROR", ABS, ROR, 6);

    op!(0x70, "BVS", REL, BVS, 2);
    op!(0x71, "ADC", IZY, ADC, 5);
    op!(0x75, "ADC", ZPX, ADC, 4);
    op!(0x76, "ROR", ZPX, ROR, 6);
    op!(0x78, "SEI", IMP, SEI, 2);
    op!(0x79, "ADC", ABY, ADC, 4);
    op!(0x7D, "ADC", ABX, ADC, 4);
    op!(0x7E, "ROR", ABX, ROR, 7);

    op!(0x81, "STA", IZX, STA, 6);
    op!(0x84, "STY", ZP0, STY, 3);
    op!(0x85, "STA", ZP0, STA, 3);
    op!(0x86, "STX", ZP0, STX, 3);
    op!(0x88, "DEY", IMP, DEY, 2);
    op!(0x8A, "TXA", IMP, TXA, 2);
    op!(0x8C, "STY", ABS, STY, 4);
    op!(0x8D, "STA", ABS, STA, 4);
    op!(0x8E, "STX", ABS, STX, 4);

    op!(0x90, "BCC", REL, BCC, 2);
    op!(0x91, "STA", IZY, STA, 6);
    op!(0x94, "STY", ZPX, STY, 4);
    op!(0x95, "STA", ZPX, STA, 4);
    op!(0x96, "STX", ZPY, STX, 4);
    op!(0x98, "TYA", IMP, TYA, 2);
    op!(0x99, "STA", ABY, STA, 5);
    op!(0x9A, "TXS", IMP, TXS, 2);
    op!(0x9D, "STA", ABX, STA, 5);

    op!(0xA0, "LDY", IMM, LDY, 2);
    op!(0xA1, "LDA", IZX, LDA, 6);
    op!(0xA2, "LDX", IMM, LDX, 2);
    op!(0xA4, "LDY", ZP0, LDY, 3);
    op!(0xA5, "LDA", ZP0, LDA, 3);
    op!(0xA6, "LDX", ZP0, LDX, 3);
    op!(0xA8, "TAY", IMP, TAY, 2);
    op!(0xA9, "LDA", IMM, LDA, 2);
    op!(0xAA, "TAX", IMP, TAX, 2);
    op!(0xAC, "LDY", ABS, LDY, 4);
    op!(0xAD, "LDA", ABS, LDA, 4);
    op!(0xAE, "LDX", ABS, LDX, 4);

    op!(0xB0, "BCS", REL, BCS, 2);
    op!(0xB1, "LDA", IZY, LDA, 5);
    op!(0xB4, "LDY", ZPX, LDY, 4);
    op!(0xB5, "LDA", ZPX, LDA, 4);
    op!(0xB6, "LDX", ZPY, LDX, 4);
    op!(0xB8, "CLV", IMP, CLV, 2);
    op!(0xB9, "LDA", ABY, LDA, 4);
    op!(0xBA, "TSX", IMP, TSX, 2);
    op!(0xBC, "LDY", ABX, LDY, 4);
    op!(0xBD, "LDA", ABX, LDA, 4);
    op!(0xBE, "LDX", ABY, LDX, 4);

    op!(0xC0, "CPY", IMM, CPY, 2);
    op!(0xC1, "CMP", IZX, CMP, 6);
    op!(0xC4, "CPY", ZP0, CPY, 3);
    op!(0xC5, "CMP", ZP0, CMP, 3);
    op!(0xC6, "DEC", ZP0, DEC, 5);
    op!(0xC8, "INY", IMP, INY, 2);
    op!(0xC9, "CMP", IMM, CMP, 2);
    op!(0xCA, "DEX", IMP, DEX, 2);
    op!(0xCC, "CPY", ABS, CPY, 4);
    op!(0xCD, "CMP", ABS, CMP, 4);
    op!(0xCE, "DEC", ABS, DEC, 6);

    op!(0xD0, "BNE", REL, BNE, 2);
    op!(0xD1, "CMP", IZY, CMP, 5);
    op!(0xD5, "CMP", ZPX, CMP, 4);
    op!(0xD6, "DEC", ZPX, DEC, 6);
    op!(0xD8, "CLD", IMP, CLD, 2);
    op!(0xD9, "CMP", ABY, CMP, 4);
    op!(0xDD, "CMP", ABX, CMP, 4);
    op!(0xDE, "DEC", ABX, DEC, 7);

    op!(0xE0, "CPX", IMM, CPX, 2);
    op!(0xE1, "SBC", IZX, SBC, 6);
    op!(0xE4, "CPX", ZP0, CPX, 3);
    op!(0xE5, "SBC", ZP0, SBC, 3);
    op!(0xE6, "INC", ZP0, INC, 5);
    op!(0xE8, "INX", IMP, INX, 2);
    op!(0xE9, "SBC", IMM, SBC, 2);
    op!(0xEA, "NOP", IMP, NOP, 2);
    op!(0xEC, "CPX", ABS, CPX, 4);
    op!(0xED, "SBC", ABS, SBC, 4);
    op!(0xEE, "INC", ABS, INC, 6);

    op!(0xF0, "BEQ", REL, BEQ, 2);
    op!(0xF1, "SBC", IZY, SBC, 5);
    op!(0xF5, "SBC", ZPX, SBC, 4);
    op!(0xF6, "INC", ZPX, INC, 6);
    op!(0xF8, "SED", IMP, SED, 2);
    op!(0xF9, "SBC", ABY, SBC, 4);
    op!(0xFD, "SBC", ABX, SBC, 4);
    op!(0xFE, "INC", ABX, INC, 7);

    t
}

pub static LOOKUP: [Instruction; 256] = build_lookup();

pub struct Olc6502 {
    // registers
    a      : u8,  // Accumulator register
    x      : u8,  // X register
    y      : u8,  // Y register
    stkp   : u8,  // Stack pointer (points to location on bus) 
    pc     : u16, // Program counter
    status : u8,  // Status register

    // internal state
    fetched  : u8, 
    addr_abs : u16,
    addr_rel : u16,
    opcode   : u8, 
    cycles   : u8,
}




impl Olc6502 {
    pub fn new() -> Self {
        Self {
            // init registers etc
            a:        0,
            x:        0,
            y:        0,
            stkp:     0,
            pc:       0,
            status:   0,

            fetched:  0, 
            addr_abs: 0, 
            addr_rel: 0, 
            opcode:   0,
            cycles:   0,
        }
    }

    pub fn read(&self, bus: &Bus, addr: u16) -> u8 {
        
        // In normal operation "read only" is set to false. This may seem odd. Some
        // devices on the bus may change state when they are read from, and this 
        // is intentional under normal circumstances. However the disassembler will
        // want to read the data at an address without changing the state of the
        // devices on the bus
        let read_only: bool = false;
        bus.read(addr, read_only)
    }

    // Writes a byte to the bus at the specified address
    pub fn write(&self, bus: &mut Bus, addr: u16, data: u8) {

        bus.write(addr, data)
    }

    
    pub fn get_registers(&self) -> (u8, u8, u8, u8, u16, u8) {
        (self.a, self.x, self.y, self.stkp, self.pc, self.status)
    }
    pub fn set_registers(&mut self, a:u8,x:u8,y:u8,s:u8,pc:u16,p:u8) {
        (self.a, self.x, self.y, self.stkp, self.pc, self.status) = (a, x, y, s, pc, p);
    }

    pub fn get_state(&self) -> (u8, u16, u16, u8, u8) {
        (self.fetched, self.addr_abs, self.addr_rel, self.opcode, self.cycles)
    }

    pub fn get_remaining_cycles(&self) -> u8 {
        self.cycles
    }

    pub fn force_cycles_zero(&mut self) {
        self.cycles = 0;
    }

    ///////////////////////////////////////////////////////////////////////////////
    // EXTERNAL INPUTS

    // Forces the 6502 into a known state. This is hard-wired inside the CPU. The
    // registers are set to 0x00, the status register is cleared except for unused
    // bit which remains at 1. An absolute address is read from location 0xFFFC
    // which contains a second address that the program counter is set to. This 
    // allows the programmer to jump to a known and programmable location in the
    // memory to start executing from. Typically the programmer would set the value
    // at location 0xFFFC at compile time.
    pub fn reset(&mut self, bus: &mut Bus) {
        self.a      = 0; 
        self.x      = 0;
        self.y      = 0; 
        self.stkp   = 0xFD; 
        self.status = 0x00 | FLAG6502_U;

        self.addr_abs = 0xFFFC;
        let lo: u16   = self.read(bus,self.addr_abs + 0) as u16;
        let hi: u16   = self.read(bus,self.addr_abs + 1) as u16;
        self.pc       = (hi << 8) | lo; 

        self.addr_rel = 0x0000;
        self.addr_abs = 0x0000;
        self.fetched  = 0x00;
        
        self.cycles  = 8;

    }

    // Interrupt requests are a complex operation and only happen if the
    // "disable interrupt" flag is 0. IRQs can happen at any time, but
    // you dont want them to be destructive to the operation of the running 
    // program. Therefore the current instruction is allowed to finish
    // (which I facilitate by doing the whole thing when cycles == 0) and 
    // then the current program counter is stored on the stack. Then the
    // current status register is stored on the stack. When the routine
    // that services the interrupt has finished, the status register
    // and program counter can be restored to how they where before it 
    // occurred. This is impemented by the "RTI" instruction. Once the IRQ
    // has happened, in a similar way to a reset, a programmable address
    // is read form hard coded location 0xFFFE, which is subsequently
    // set to the program counter.
    pub fn irq(&mut self, bus: &mut Bus) {
        if self.get_flag(FLAG6502_I) != 0 {
            self.nmi(bus);
        }
    }

    // A Non-Maskable Interrupt cannot be ignored. It behaves in exactly the
    // same way as a regular IRQ, but reads the new program counter address
    // form location 0xFFFA.
    pub fn nmi(&mut self, bus: &mut Bus) {
        self.write(bus, 0x0100 + self.stkp as u16, ((self.pc >> 8) & 0x00FF) as u8);
        self.stkp = self.stkp.wrapping_sub(1); 
        self.write(bus, 0x0100 + self.stkp as u16, ((self.pc     ) & 0x00FF) as u8);
        self.stkp = self.stkp.wrapping_sub(1); 

        self.set_flag(FLAG6502_B, false);
        self.set_flag(FLAG6502_U, true);
        self.set_flag(FLAG6502_I, true);

        self.write(bus, 0x0100 + self.stkp as u16, self.status);
        self.stkp = self.stkp.wrapping_sub(1); 

        self.addr_abs = 0xFFFE;
        let lo: u16 = self.read(bus,self.addr_abs + 0) as u16;
        let hi: u16 = self.read(bus,self.addr_abs + 1) as u16;
        self.pc = (hi << 8) | lo; 
        
        self.cycles = 7;
    }


    // Each instruction requires a variable number of clock cycles to execute.
    // In my emulation, I only care about the final result and so I perform
    // the entire computation in one hit. In hardware, each clock cycle would
    // perform "microcode" style transformations of the CPUs state.
    //
    // To remain compliant with connected devices, it's important that the 
    // emulation also takes "time" in order to execute instructions, so I
    // implement that delay by simply counting down the cycles required by 
    // the instruction. When it reaches 0, the instruction is complete, and
    // the next one is ready to be executed.
    pub fn clock(&mut self, bus: &mut Bus) {
    
        // Only actually do work once enough time has passed
        if self.cycles == 0 {
            // Read one byte from bus containing the opcode
            self.opcode = bus.read(self.pc, true);
            self.set_flag(FLAG6502_U, true);
            self.pc = self.pc.wrapping_add(1);

            let inst = LOOKUP[self.opcode as usize];
            self.cycles = inst.cycles;
            
            // addressing mode
            let additional_cycle1 = match inst.addrmode {
                AddressMode::IMP => self.imp(bus),
                AddressMode::IMM => self.imm(bus),
                AddressMode::ZP0 => self.zp0(bus),
                AddressMode::ZPX => self.zpx(bus),
                AddressMode::ZPY => self.zpy(bus),
                AddressMode::ABS => self.abs(bus),
                AddressMode::ABX => self.abx(bus),
                AddressMode::ABY => self.aby(bus),
                AddressMode::IND => self.ind(bus),
                AddressMode::IZX => self.izx(bus),
                AddressMode::IZY => self.izy(bus),
                AddressMode::REL => self.rel(bus),
            };

            let additional_cycle2 = match inst.operation {
                // System
                Operation::BRK => self.brk(bus),
                Operation::NOP => self.nop(bus),

                // Loads
                Operation::LDA => self.lda(bus),
                Operation::LDX => self.ldx(bus),
                Operation::LDY => self.ldy(bus),

                // Stores
                Operation::STA => self.sta(bus),
                Operation::STX => self.stx(bus),
                Operation::STY => self.sty(bus),

                // Register transfers
                Operation::TAX => self.tax(bus),
                Operation::TAY => self.tay(bus),
                Operation::TXA => self.txa(bus),
                Operation::TYA => self.tya(bus),
                Operation::TSX => self.tsx(bus),
                Operation::TXS => self.txs(bus),

                // Stack
                Operation::PHA => self.pha(bus),
                Operation::PHP => self.php(bus),
                Operation::PLA => self.pla(bus),
                Operation::PLP => self.plp(bus),

                // Logical
                Operation::AND => self.and(bus),
                Operation::EOR => self.eor(bus),
                Operation::ORA => self.ora(bus),
                Operation::BIT => self.bit(bus),

                // Arithmetic / Compare
                Operation::ADC => self.adc(bus),
                Operation::SBC => self.sbc(bus),
                Operation::CMP => self.cmp(bus),
                Operation::CPX => self.cpx(bus),
                Operation::CPY => self.cpy(bus),

                // Inc / Dec
                Operation::INC => self.inc(bus),
                Operation::INX => self.inx(bus),
                Operation::INY => self.iny(bus),
                Operation::DEC => self.dec(bus),
                Operation::DEX => self.dex(bus),
                Operation::DEY => self.dey(bus),

                // Shifts / Rotates
                Operation::ASL => self.asl(bus),
                Operation::LSR => self.lsr(bus),
                Operation::ROL => self.rol(bus),
                Operation::ROR => self.ror(bus),

                // Jumps / Calls
                Operation::JMP => self.jmp(bus),
                Operation::JSR => self.jsr(bus),
                Operation::RTS => self.rts(bus),
                Operation::RTI => self.rti(bus),

                // Branches
                Operation::BCC => self.bcc(bus),
                Operation::BCS => self.bcs(bus),
                Operation::BEQ => self.beq(bus),
                Operation::BMI => self.bmi(bus),
                Operation::BNE => self.bne(bus),
                Operation::BPL => self.bpl(bus),
                Operation::BVC => self.bvc(bus),
                Operation::BVS => self.bvs(bus),

                // Flag operations
                Operation::CLC => self.clc(bus),
                Operation::CLD => self.cld(bus),
                Operation::CLI => self.cli(bus),
                Operation::CLV => self.clv(bus),
                Operation::SEC => self.sec(bus),
                Operation::SED => self.sed(bus),
                Operation::SEI => self.sei(bus),

                // Illegal / placeholder
                Operation::XXX => self.xxx(bus),
            };
            self.cycles += additional_cycle1 & additional_cycle2;

        }

        self.cycles -= 1;
        
    }



    pub fn step_instruction(&mut self, bus: &mut Bus) {
        loop {
            self.clock(bus);
            if !(self.cycles > 0) {
                break;
            }
        } 
    }


    // Returns the value of a specific bit of the status register
    pub fn get_flag(&self, f: u8) -> u8 {
        if (self.status & f) != 0 { 1 } else { 0 }
    }

    // Sets or clears a specific bit of the status register
    pub fn set_flag(&mut self, f: u8, v: bool) {
        if v {
            self.status |= f;
        } else {
            self.status &= !f;
        }
    }

    
	// Addressing Modes =============================================
	// The 6502 has a variety of addressing modes to access data in 
	// memory, some of which are direct and some are indirect (like
	// pointers in C++). Each opcode contains information about which
	// addressing mode should be employed to facilitate the 
	// instruction, in regards to where it reads/writes the data it
	// uses. The address mode changes the number of bytes that
	// makes up the full instruction, so we implement addressing
	// before executing the instruction, to make sure the program
	// counter is at the correct location, the instruction is
	// primed with the addresses it needs, and the number of clock
	// cycles the instruction requires is calculated. These functions
	// may adjust the number of cycles required depending upon where
	// and how the memory is accessed, so they return the required
	// adjustment.

    // Address Mode: Implied
    // There is no additional data required for this instruction. The instruction
    // does something very simple like like sets a status bit. However, we will
    // target the accumulator, for instructions like PHA
    fn imp(&mut self, _bus: &mut Bus) -> u8 { 
        self.fetched = self.a; 
        0
    }

    // Address Mode: Immediate
    // The instruction expects the next byte to be used as a value, so we'll prep
    // the read address to point to the next byte
    fn imm(&mut self, _bus: &mut Bus) -> u8 { 
        self.addr_abs = self.pc; 
        self.pc = self.pc.wrapping_add(1);
        0
     }
    // Zero-page addressing
    // Pages are a conceptual way of organising memory
    // 0xFF55: A 16-bit address consists of two 8-bit bytes
    // High byte: page
    // Low byte: offset
    // We can think of the address space as 256 pages of 256 bytes
    // Zero-page: The effective address is between 0x0000 and 0x00FF.
    fn zp0(&mut self, bus: &mut Bus) -> u8 { 
        self.addr_abs  = self.read(bus, self.pc) as u16;
        self.pc        = self.pc.wrapping_add(1);
        self.addr_abs &= 0x00FF; // Clear upper bits if addr_abs is not within range
        0
     }
     // Zero-page offset with x-register addressing
    fn zpx(&mut self, bus: &mut Bus) -> u8 { 
        self.addr_abs  = self.read(bus, self.pc) as u16;
        self.addr_abs  = self.addr_abs.wrapping_add(self.x as u16);
        self.pc        = self.pc.wrapping_add(1);
        self.addr_abs &= 0x00FF; // Clear upper bits in case addr_abs not within range
        0
    }
     // Zero-page offset with y-register addressing
    fn zpy(&mut self, bus: &mut Bus) -> u8 { 
        self.addr_abs  = self.read(bus, self.pc) as u16;
        self.addr_abs  = self.addr_abs.wrapping_add(self.y as u16);
        self.pc        = self.pc.wrapping_add(1);
        self.addr_abs &= 0x00FF; // Clear upper bits in case addr_abs not within range
        0
    }

    // Address Mode: Relative
    // This address mode is exclusive to branch instructions. The address
    // must reside within -128 to +127 of the branch instruction, i.e.
    // you cant directly branch to any address in the addressable range.
    fn rel(&mut self, bus: &mut Bus) -> u8 { 
        
        self.addr_rel = self.read(bus, self.pc) as u16;
        self.pc       = self.pc.wrapping_add(1);

        if self.addr_rel & 0x80 != 0 {
            self.addr_rel |= 0xFF00;
        }
        0
    }
    // Stick together two 8-bit values to form a 16-bit address
    fn abs(&mut self, bus: &mut Bus) -> u8 { 
        let lo : u16  = self.read(bus, self.pc) as u16;
        self.pc       = self.pc.wrapping_add(1);
        let hi : u16  = self.read(bus, self.pc) as u16;
        self.pc       = self.pc.wrapping_add(1);
        self.addr_abs = (hi << 8) | lo; 
        0
    }
    
    // Address Mode: Absolute with X Offset
    // Fundamentally the same as absolute addressing, but the contents of the X Register
    // is added to the supplied two byte address. If the resulting address changes
    // the page, an additional clock cycle is required
    fn abx(&mut self, bus: &mut Bus) -> u8 { 
        let lo : u16   = self.read(bus, self.pc) as u16;
        self.pc        = self.pc.wrapping_add(1);
        let hi : u16   = self.read(bus, self.pc) as u16;
        self.pc        = self.pc.wrapping_add(1);
        self.addr_abs  = (hi << 8) | lo; 

        self.addr_abs += self.x as u16;

        // If the whole address has changed to a different page, we may need one more clock cycle
        // Overflow: Carry bit from the low byte has carried into the high byt
        if (self.addr_abs & 0xFF00) != (hi << 8) {
            1
        } else {
            0
        }
    }
    
    // Address Mode: Absolute with Y Offset
    // Fundamentally the same as absolute addressing, but the contents of the Y Register
    // is added to the supplied two byte address. If the resulting address changes
    // the page, an additional clock cycle is required
    fn aby(&mut self, bus: &mut Bus) -> u8 {
        let lo : u16   = self.read(bus, self.pc) as u16;
        self.pc        = self.pc.wrapping_add(1);
        let hi : u16   = self.read(bus, self.pc) as u16;
        self.pc        = self.pc.wrapping_add(1);
        self.addr_abs  = (hi << 8) | lo; 

        self.addr_abs += self.y as u16;

        // If the whole address has changed to a different page, we may need one more clock cycle
        // Overflow: Carry bit from the low byte has carried into the high byt
        if (self.addr_abs & 0xFF00) != (hi << 8) {
            1
        } else {
            0
        }
    }
    // 6502-way of implementing pointers: Indirect addressing
    // Address supplied with the operation is a pointer
    // We need to read the bus at that address to get the actual address where the data we want resides
    // But there is a processor bug: https://www.nesdev.com/6502bugs.txt
    // An indirect JMP (xxFF) will fail because the MSB will be fetched from
    // address xx00 instead of page xx+1.
    fn ind(&mut self, bus: &mut Bus) -> u8 { 

        let lo : u16   = self.read(bus, self.pc) as u16;
        self.pc        = self.pc.wrapping_add(1);
        let hi : u16   = self.read(bus, self.pc) as u16;
        self.pc        = self.pc.wrapping_add(1);

        let ptr: u16   = (hi << 8) | lo; 
        // Simulate page boundary hardware bug
        if lo == 0x00FF { 
            self.addr_abs  = (self.read(bus, ptr & 0xFF00) as u16) << 8 | (self.read(bus, ptr + 0) as u16); 
        } else {
            self.addr_abs  = (self.read(bus, ptr + 1)      as u16) << 8 | (self.read(bus, ptr + 0) as u16); 
        }
        0
    }

    // Address Mode: Indirect X
    // The supplied 8-bit address is offset by X Register to index
    // a location in page 0x00. The actual 16-bit address is read 
    // from this location
    fn izx(&mut self, bus: &mut Bus) -> u8 { 
        
        let t : u16      = self.read(bus, self.pc) as u16;
        self.pc          = self.pc.wrapping_add(1);
        
        let zp_ptr: u16  = (t + self.x as u16) & 0x00FF;
        let lo: u16      = self.read(bus, zp_ptr) as u16;
        let hi: u16      = self.read(bus, (zp_ptr + 1) & 0x00FF) as u16;

        self.addr_abs    = (hi << 8) | lo; 
        
        0
    }
    

    // Address Mode: Indirect Y
    // The supplied 8-bit address indexes a location in page 0x00. From 
    // here the actual 16-bit address is read, and the contents of
    // Y Register is added to it to offset it. If the offset causes a
    // change in page then an additional clock cycle is required.
    fn izy(&mut self, bus: &mut Bus) -> u8 { 
        let t : u16    = self.read(bus, self.pc) as u16;
        self.pc        = self.pc.wrapping_add(1);
        let lo : u16   = self.read(bus, (t    ) & 0x00FF) as u16;
        let hi : u16   = self.read(bus, (t + 1) & 0x00FF) as u16; 

        self.addr_abs  = (hi << 8) | lo; 
        self.addr_abs  = self.addr_abs.wrapping_add(self.y as u16); 

        if (self.addr_abs & 0xFF00) != (hi << 8) {
            1
        } else {
            0
        }
     }

     


    // This function sources the data used by the instruction into 
    // a convenient numeric variable. Some instructions dont have to 
    // fetch data as the source is implied by the instruction. For example
    // "INX" increments the X register. There is no additional data
    // required. For all other addressing modes, the data resides at 
    // the location held within addr_abs, so it is read from there. 
    // Immediate adress mode exploits this slightly, as that has
    // set addr_abs = pc + 1, so it fetches the data from the
    // next byte for example "LDA $FF" just loads the accumulator with
    // 256, i.e. no far reaching memory fetch is required. "fetched"
    // is a variable global to the CPU, and is set by calling this 
    // function. It also returns it for convenience.
    pub fn fetch(&mut self, bus: &mut Bus) -> u8 {
        let inst = LOOKUP[self.opcode as usize];

        if inst.addrmode != AddressMode::IMP {
            self.fetched = self.read(bus, self.addr_abs);
        }

        self.fetched
    }


    // This function captures illegal opcodes
    fn xxx(&mut self, _bus: &mut Bus) -> u8 { 0 }

    // Addition!
    // Add data fetched from memory to accumulator, including the carry bit
    // A += M + C
    // A = 250
    // M = 10 - answer is 4 + carry bit
    // We can carry out 16-bit addition despite only working with 8-bit numbers by taking two 8-bit variables together
    // This way we can work with arbitrarily high-precision numbers
    // But signed numbers are a different story
    // 1000100 = 128 + 4 = 132 (0-255) 
    // 132 -> -124 (-128-127, overflow) 
    // Sign = first bit
    // 10000100 = 132 or -124
    //+00010001 =  17 or   17
    // ______________________
    // 10010101 = 149 or -107
    // P + P = P
    // P + P = N - Overflow
    // P + N = Cant overflow
    // N + N = N
    // N + N = P - Overflow
    // V register = Was there an overflow? 
    // In the below: most significant bits (0 = positive, 1 = negative)
    // A M R = V
    // 0 0 0   0 
    // 0 0 1   1
    // 0 1 0   0
    // 0 1 1   0
    // 1 0 0   0
    // 1 0 1   0 
    // 1 1 0   1
    // 1 1 1   0
    fn adc(&mut self, bus: &mut Bus) -> u8 { 
        self.fetch(bus);
        let temp: u16 = self.a as u16 + self.fetched as u16 + self.get_flag(FLAG6502_C) as u16; 
        self.set_flag(FLAG6502_C, temp > 255);
        self.set_flag(FLAG6502_Z, (temp & 0x00FF) == 0); // 
        self.set_flag(FLAG6502_N, (temp & 0x80) == 0);   // Check the most significant bit
        self.set_flag(FLAG6502_B,  (!((self.a as u16) ^ (self.fetched as u16)) & ((self.a as u16) ^ (temp as u16)) & 0x0080) != 0);

        self.a = (temp & 0x00FF) as u8; 
        1 // can require an additional clock cycle
    }

    // Instruction: Bitwise Logic AND
    // Function:    A = A & M
    // Flags Out:   N, Z
    fn and(&mut self, bus: &mut Bus) -> u8 { 
        self.fetch(bus);
        self.a = self.a & self.fetched; 
        self.set_flag(FLAG6502_Z, self.a == 0x00);
        self.set_flag(FLAG6502_N, self.a &  0x80 != 0);
        1
    }
    
    // Instruction: Arithmetic Shift Left
    // Function:    A = C <- (A << 1) <- 0
    // Flags Out:   N, Z, C
    fn asl(&mut self, bus: &mut Bus) -> u8 { 
        self.fetch(bus);
        let temp: u16 = (self.fetched as u16) << 1; 
        self.set_flag(FLAG6502_C, (temp & 0xFF00) > 0);
        self.set_flag(FLAG6502_Z, (temp & 0x00FF) == 0x00);
        self.set_flag(FLAG6502_N, (temp & 0x80)   > 0);
        
        let inst = LOOKUP[self.opcode as usize];
        
        if inst.addrmode == AddressMode::IMP {
            self.a = (temp & 0x00FF) as u8;
        } else {
            self.write(bus, self.addr_abs, (temp & 0x00FF) as u8); 
        }
        0
     }

     

    // A & memory
    // BIT modifies flags, but does not change memory or registers. The zero flag is set depending on the result of the accumulator AND memory value, effectively applying a bitmask and then checking if any bits are set. Bits 7 and 6 of the memory value are loaded directly into the negative and overflow flags, allowing them to be easily checked without having to load a mask into A.
    // Because BIT only changes CPU flags, it is sometimes used to trigger the read side effects of a hardware register without clobbering any CPU registers, or even to waste cycles as a 3-cycle NOP. As an advanced trick, it is occasionally used to hide a 1- or 2-byte instruction in its operand that is only executed if jumped to directly, allowing two code paths to be interleaved. However, because the instruction in the operand is treated as an address from which to read, this carries risk of triggering side effects if it reads a hardware register. This trick can be useful when working under tight constraints on space, time, or register usage. 
    fn bit(&mut self, bus: &mut Bus) -> u8 { 
        self.fetch(bus);
        let temp: u16 = (self.a & self.fetched) as u16; 
        self.set_flag(FLAG6502_Z, (temp & 0xFF00) == 0x00);
        self.set_flag(FLAG6502_N, self.fetched & (1 << 7) > 0);
        self.set_flag(FLAG6502_V, self.fetched & (1 << 6) > 0);
        0
    }

    
    // Trigger an interrupt request in software
    // Push current program counter and process flags to the stack
    // Set interrupt disable flag and jump to IRQ handler
    fn brk(&mut self, bus: &mut Bus) -> u8 {
        self.pc = self.pc.wrapping_add(1);
        self.write(bus, 0x0100 + self.stkp as u16, ((self.pc >> 8) & 0x00FF) as u8);
        self.stkp = self.stkp.wrapping_sub(1); 
        self.write(bus, 0x0100 + self.stkp as u16, ((self.pc     ) & 0x00FF) as u8);
        self.stkp = self.stkp.wrapping_sub(1); 

        self.set_flag(FLAG6502_B, true);

        self.write(bus, 0x0100 + self.stkp as u16, self.status);
        self.stkp = self.stkp.wrapping_sub(1); 

        self.set_flag(FLAG6502_B, false);
        self.set_flag(FLAG6502_I, true);

        self.addr_abs = 0xFFFE;
        let lo: u16 = self.read(bus,self.addr_abs + 0) as u16;
        let hi: u16 = self.read(bus,self.addr_abs + 1) as u16;
        self.pc = (hi << 8) | lo; 
        
        0
     }

    

     // helper function to implement branching
     // Consumes 1 or 2 cycles and updates pc to pc + addr_rel
    fn branch(&mut self, bus: &mut Bus) {
        self.cycles   = self.cycles.wrapping_add(1);
        self.addr_abs = self.pc.wrapping_add(self.addr_rel); 

        // second cycle of clock penalty if we cross a page boundary
        if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
            self.cycles = self.cycles.wrapping_add(1); 
        }
        self.pc = self.addr_abs;
    }
    
    // Instruction: Branch if Carry Clear
    // Function:    if(C == 0) pc = address 
    fn bcc(&mut self, bus: &mut Bus) -> u8 { 
        if self.get_flag(FLAG6502_C) == 0 {
            self.branch(bus);
        }
        0
    }

    // Instruction: Branch if Carry Set
    // Function:    if(C == 1) pc = address
    fn bcs(&mut self, bus: &mut Bus) -> u8 { 
        if self.get_flag(FLAG6502_C) == 1 {
            self.branch(bus);
        }
        0
    }
    
    // Instruction: Branch if Equal
    // Function:    if(Z == 1) pc = address
    fn beq(&mut self, bus: &mut Bus) -> u8 { 
        if self.get_flag(FLAG6502_Z) == 1 {
            self.branch(bus);
        }
        0
    }
    // Instruction: Branch if Negative
    // Function:    if(N == 1) pc = address
    fn bmi(&mut self, bus: &mut Bus) -> u8 { 
        if self.get_flag(FLAG6502_N) == 1 {
            self.branch(bus);
        }
        0
    }
    // Instruction: Branch if Not Equal
    // Function:    if(Z == 0) pc = address
    fn bne(&mut self, bus: &mut Bus) -> u8 { 
        if self.get_flag(FLAG6502_Z) == 0 {
            self.branch(bus);
        }
        0
    }
    // Instruction: Branch if Positive
    // Function:    if(N == 0) pc = address
    fn bpl(&mut self, bus: &mut Bus) -> u8 {
        if self.get_flag(FLAG6502_N) == 0 {
            self.branch(bus);
        }
        0
    }

    // Branch if overflow clear - set pc to pc + addr_rel
    fn bvc(&mut self, bus: &mut Bus) -> u8 { 
        if self.get_flag(FLAG6502_V) == 0 {
            self.branch(bus);
        }
        0
    }
    // Branch if overflow set - set pc to pc + addr_rel
    fn bvs(&mut self, bus: &mut Bus) -> u8 { 
        if self.get_flag(FLAG6502_V) == 1 {
            self.branch(bus);
        }
        0
     }

    // Instruction: Clear Carry Flag
    // Function:    C = 0
    fn clc(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(FLAG6502_C, false);
        0
    }
    
    // Instruction: Clear Decimal Flag
    // Function:    D = 0
    fn cld(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(FLAG6502_D, false);
        0
    }

    
    // Instruction: Disable Interrupts / Clear Interrupt Flag
    // Function:    I = 0
    fn cli(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(FLAG6502_I, false);
        0
    }

        
    // Instruction: Clear Overflow Flag
    // Function:    V = 0
    fn clv(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(FLAG6502_V, false);
        0
    }

    
    // Instruction: Compare Accumulator
    // Function:    C <- A >= M      Z <- (A - M) == 0
    // Flags Out:   N, C, Z
    fn cmp(&mut self, bus: &mut Bus) -> u8 { 
        self.fetch(bus);
        // keep the borrow bit information by computing as u16
        let temp: u16 = (self.a as u16) - (self.fetched  as u16); 
        self.set_flag(FLAG6502_C, self.a >= self.fetched);
        self.set_flag(FLAG6502_Z, temp & 0x00FF == 0x0000);
        self.set_flag(FLAG6502_N, temp & 0x0080 != 0x0000);
        1
     }

    // Instruction: Compare X Register
    // Function:    C <- X >= M      Z <- (X - M) == 0
    // Flags Out:   N, C, Z
    fn cpx(&mut self, bus: &mut Bus) -> u8 { 
        self.fetch(bus);
        let temp: u16 = (self.x as u16) - (self.fetched  as u16); 
        self.set_flag(FLAG6502_C, self.x >= self.fetched);
        self.set_flag(FLAG6502_Z, temp & 0x00FF == 0x0000);
        self.set_flag(FLAG6502_N, temp & 0x0080 != 0x0000);
        1

    }

    
    // Instruction: Compare Y Register
    // Function:    C <- Y >= M      Z <- (Y - M) == 0
    // Flags Out:   N, C, Z
    fn cpy(&mut self, bus: &mut Bus) -> u8 { 
        self.fetch(bus);
        let temp: u16 = (self.y as u16) - (self.fetched  as u16); 
        self.set_flag(FLAG6502_C, self.y >= self.fetched);
        self.set_flag(FLAG6502_Z, temp & 0x00FF == 0x0000);
        self.set_flag(FLAG6502_N, temp & 0x0080 != 0x0000);
        1
     }

    // Instruction: Bitwise Logic XOR
    // Function:    A = A xor M
    // Flags Out:   N, Z
    fn eor(&mut self, bus: &mut Bus) -> u8 { 
        self.fetch(bus);
        self.a = self.a ^ self.fetched; 
        self.set_flag(FLAG6502_Z, self.a        == 0x00);
        self.set_flag(FLAG6502_N, self.a & 0x80 != 0x00);
        1
    }

    
    // Instruction: Bitwise Logic OR
    // Function:    A = A | M
    // Flags Out:   N, Z
    fn ora(&mut self, bus: &mut Bus) -> u8 { 
        self.fetch(bus);
        self.a = self.a | self.fetched; 
        self.set_flag(FLAG6502_Z, self.a        == 0x00);
        self.set_flag(FLAG6502_N, self.a & 0x80 != 0x00);
        1
    }

     
    // Instruction: Decrement Value at Memory Location
    // Function:    M = M - 1
    // Flags Out:   N, Z
    fn dec(&mut self, bus: &mut Bus) -> u8 { 
        self.fetch(bus);
        let temp: u8 = self.fetched.wrapping_sub(1);
        self.write(bus, self.addr_abs, temp);
        self.set_flag(FLAG6502_Z, temp        == 0x00);
        self.set_flag(FLAG6502_N, temp & 0x80 != 0x00);
        0
    }

    // Instruction: Decrement X Register
    // Function:    X = X - 1
    // Flags Out:   N, Z
    fn dex(&mut self, bus: &mut Bus) -> u8 { 
        self.x = self.x.wrapping_sub(1);
        self.set_flag(FLAG6502_Z, self.x        == 0x00);
        self.set_flag(FLAG6502_N, self.x & 0x80 != 0x00);
        0
    }
    
    // Instruction: Decrement Y Register
    // Function:    Y = Y - 1
    // Flags Out:   N, Z
    fn dey(&mut self, bus: &mut Bus) -> u8 { 
        self.y = self.y.wrapping_sub(1);
        self.set_flag(FLAG6502_Z, self.y        == 0x00);
        self.set_flag(FLAG6502_N, self.y & 0x80 != 0x00);
        0
    }

    // Instruction: Increment Value at Memory Location
    // Function:    M = M + 1
    // Flags Out:   N, Z
    fn inc(&mut self, bus: &mut Bus) -> u8 { 
        self.fetch(bus);
        let temp: u8 = self.fetched.wrapping_add(1);
        self.write(bus, self.addr_abs, temp);
        self.set_flag(FLAG6502_Z, temp        == 0x00);
        self.set_flag(FLAG6502_N, temp & 0x80 != 0x00);
        0 
    }


    // Instruction: Increment X Register
    // Function:    X = X + 1
    // Flags Out:   N, Z
    fn inx(&mut self, bus: &mut Bus) -> u8 { 
        self.x = self.x.wrapping_add(1);
        self.set_flag(FLAG6502_Z, self.x        == 0x00);
        self.set_flag(FLAG6502_N, self.x & 0x80 != 0x00);
        0
    }

    // Instruction: Increment Y Register
    // Function:    Y = Y + 1
    // Flags Out:   N, Z
    fn iny(&mut self, bus: &mut Bus) -> u8 { 
        self.y = self.y.wrapping_add(1);
        self.set_flag(FLAG6502_Z, self.y        == 0x00);
        self.set_flag(FLAG6502_N, self.y & 0x80 != 0x00);
        0
    }

    
    // Instruction: Jump To Location
    // Function:    pc = address
    fn jmp(&mut self, _bus: &mut Bus) -> u8 { 
        self.pc = self.addr_abs;
        0
    }
        
    // Instruction: Jump To Sub-Routine
    // Function:    Push current pc to stack, pc = address
    fn jsr(&mut self, bus: &mut Bus) -> u8 { 
        
        self.pc = self.pc.wrapping_sub(1);
        self.write(bus, 0x0100 + self.stkp as u16, ((self.pc >> 8) & 0x00FF) as u8);
        self.stkp = self.stkp.wrapping_sub(1); 
        self.write(bus, 0x0100 + self.stkp as u16, ((self.pc     ) & 0x00FF) as u8);
        self.stkp = self.stkp.wrapping_sub(1); 

        self.pc = self.addr_abs; 
        0
    }

    // Instruction: Load The Accumulator
    // Function:    A = M
    // Flags Out:   N, Z
    fn lda(&mut self, bus: &mut Bus) -> u8 { 
        self.fetch(bus);
        self.a = self.fetched;
        self.set_flag(FLAG6502_Z, self.a        == 0x00);
        self.set_flag(FLAG6502_N, self.a & 0x80 != 0x00);
        1
     }
    
    // Instruction: Load The X Register
    // Function:    X = M
    // Flags Out:   N, Z
    fn ldx(&mut self, bus: &mut Bus) -> u8 { 
        self.fetch(bus);
        self.x = self.fetched;
        self.set_flag(FLAG6502_Z, self.x        == 0x00);
        self.set_flag(FLAG6502_N, self.x & 0x80 != 0x00);
        1
    }

    
    // Instruction: Load The Y Register
    // Function:    Y = M
    // Flags Out:   N, Z
    fn ldy(&mut self, bus: &mut Bus) -> u8 { 
        self.fetch(bus);
        self.y = self.fetched;
        self.set_flag(FLAG6502_Z, self.y        == 0x00);
        self.set_flag(FLAG6502_N, self.y & 0x80 != 0x00);
        1
    }

    fn lsr(&mut self, bus: &mut Bus) -> u8 {
        self.fetch(bus);
        self.set_flag(FLAG6502_C, self.fetched & 0x01 != 0x00);
        self.set_flag(FLAG6502_Z, self.y              == 0x00);
        self.set_flag(FLAG6502_N, self.y & 0x80       != 0x00);        
    
        let temp : u8 = (self.fetched >> 1) as u8;	
        let inst = LOOKUP[self.opcode as usize];

        if inst.addrmode != AddressMode::IMP {
            self.a = temp;
        } else {
            self.write(bus, self.addr_abs, temp);
        }

        return 0;

    }

    // No operation codes based on https://wiki.nesdev.com/w/index.php/CPU_unofficial_opcodes
    fn nop(&mut self, _bus: &mut Bus) -> u8 { 
        match self.opcode {
            0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => 1,
            _ => 0,
        }
    }


    // Push accumulator to the stack
    // 0x0100 is hard-coded stack-location
    fn pha(&mut self, bus: &mut Bus) -> u8 {
        self.write(bus, 0x0100 + (self.stkp as u16), self.a);
        self.stkp = self.stkp.wrapping_sub(1);
        0
    }
    

    // Instruction: Push Status Register to Stack
    // Function:    status -> stack
    // Note:        Break flag is set to 1 before push
    fn php(&mut self, bus: &mut Bus) -> u8 {
        self.write(bus, 0x0100 + (self.stkp as u16), self.status | FLAG6502_B | FLAG6502_U);
        self.set_flag(FLAG6502_B, false); 
        // self.set_flag(FLAG6502_U, false);  Comment this out compared to Javid9x' implementation to satisfy Harte
        self.stkp = self.stkp.wrapping_sub(1);
        0
    }
    // Pop from stack
    // 0x0100 is hard-coded stack-location
    fn pla(&mut self, bus: &mut Bus) -> u8 { 
        self.stkp = self.stkp.wrapping_add(1); 
        self.a = self.read(bus, (0x0100 as u16) + (self.stkp as u16));
        self.set_flag(FLAG6502_Z, self.a == 0x00); 
        self.set_flag(FLAG6502_N, self.a & 0x80 != 0); 
        0
    }
    
    // Instruction: Pop Status Register off Stack
    // Function:    Status <- stack
    fn plp(&mut self, bus: &mut Bus) -> u8 { 
        self.stkp = self.stkp.wrapping_add(1); 
        self.status = self.read(bus, 0x0100 + self.stkp as u16); 
        self.set_flag(FLAG6502_U, true); 
        0
    }

    fn rol(&mut self, bus: &mut Bus) -> u8 { 
        self.fetch(bus);
        let temp = ((self.fetched as u16) << 1 ) | (self.get_flag(FLAG6502_C) as u16);
        self.set_flag(FLAG6502_C,  temp & 0xFF00  != 0x0000);
        self.set_flag(FLAG6502_Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(FLAG6502_N,  temp & 0x0080  != 0x0000);

        let inst = LOOKUP[self.opcode as usize];

        if inst.addrmode != AddressMode::IMP {
            self.a = (temp & 0x00FF) as u8;
        } else {
            self.write(bus, self.addr_abs, (temp & 0x00FF) as u8);
        }
        0
    }

    fn ror(&mut self, bus: &mut Bus) -> u8 { 
        self.fetch(bus);
        let temp = ((self.fetched as u16) >> 1 ) | ((self.get_flag(FLAG6502_C) << 7) as u16);
        self.set_flag(FLAG6502_C,  self.fetched & 0x01  != 0x00);
        self.set_flag(FLAG6502_Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(FLAG6502_N,  temp & 0x0080  != 0x0000);

        let inst = LOOKUP[self.opcode as usize];

        if inst.addrmode != AddressMode::IMP {
            self.a = (temp & 0x00FF) as u8;
        } else {
            self.write(bus, self.addr_abs, (temp & 0x00FF) as u8);
        }
        0
    }

    fn rti(&mut self, bus: &mut Bus) -> u8 { 
        self.stkp = self.stkp.wrapping_add(1); 
        self.status = self.read(bus, (0x0100 as u16) + (self.stkp as u16));
        self.status &= !FLAG6502_B;
        self.status &= !FLAG6502_U;
        self.stkp = self.stkp.wrapping_add(1); 
        
        self.pc    = self.read(bus, 0x0100 + (self.stkp as u16)) as u16;
        self.stkp = self.stkp.wrapping_add(1); 
        self.pc   |= (self.read(bus, 0x0100 + (self.stkp as u16)) as u16) << 8;
        self.stkp = self.stkp.wrapping_add(1); 
        0
    }
    fn rts(&mut self, bus: &mut Bus) -> u8 { 
        
        self.pc    = self.read(bus, 0x0100 + (self.stkp as u16)) as u16;
        self.stkp = self.stkp.wrapping_add(1); 
        self.pc   |= (self.read(bus, 0x0100 + (self.stkp as u16)) as u16) << 8;
        self.stkp = self.stkp.wrapping_add(1); 

        self.pc = self.pc.wrapping_add(1);
        0
     }
    // Subtraction = A = A - M - (1 - C)
    // A = A + -M + 1 + c
    // 5 = 00000101
    //-5 = 11111010 + 00000001
    // implement like addition
    fn sbc(&mut self, bus: &mut Bus) -> u8 {
        self.fetch(bus);
        let value : u16 = (self.fetched as u16) ^ 0x00FF;
        let temp: u16 = self.a as u16 + self.fetched as u16 + self.get_flag(FLAG6502_C) as u16; 
        self.set_flag(FLAG6502_C, temp > 255);
        self.set_flag(FLAG6502_Z, (temp & 0x00FF) == 0); // 
        self.set_flag(FLAG6502_N, (temp & 0x80) == 0);   // Check the most significant bit
        self.set_flag(FLAG6502_B,  (!((self.a as u16) ^ (self.fetched as u16)) & ((self.a as u16) ^ (temp as u16)) & 0x0080) != 0);

        self.a = (temp & 0x00FF) as u8; 
        1 // can require an additional clock cycle
    }
    

    // Instruction: Set Carry Flag
    // Function:    C = 1
    fn sec(&mut self, _bus: &mut Bus) -> u8 { 
        self.set_flag(FLAG6502_C, true);
        0
     }
    

    // Instruction: Set Decimal Flag
    // Function:    D = 1
    fn sed(&mut self, _bus: &mut Bus) -> u8 { 
        self.set_flag(FLAG6502_D, true);
        0
    }
    
    // Instruction: Set Interrupt Flag / Enable Interrupts
    // Function:    I = 1
    fn sei(&mut self, _bus: &mut Bus) -> u8 { 
        self.set_flag(FLAG6502_I, true);
        0
     }

    // Instruction: Store Accumulator at Address
    // Function:    M = A
    fn sta(&mut self, bus: &mut Bus) -> u8 { 
        self.write(bus, self.addr_abs, self.a);
        0
     }
    
    // Instruction: Store X Register at Address
    // Function:    M = X
    fn stx(&mut self, bus: &mut Bus) -> u8 { 
        self.write(bus, self.addr_abs, self.x);
        0
     }
    
    // Instruction: Store Y Register at Address
    // Function:    M = Y
    fn sty(&mut self, bus: &mut Bus) -> u8 {
        self.write(bus, self.addr_abs, self.y);
        0
     }


    // Instruction: Transfer Accumulator to X Register
    // Function:    X = A
    // Flags Out:   N, Z
    fn tax(&mut self, _bus: &mut Bus) -> u8 { 
        self.x = self.a;
        self.set_flag(FLAG6502_Z, self.x        == 0x00); 
        self.set_flag(FLAG6502_N, self.x & 0x80 != 0x00); 
        0
    }
    
    // Instruction: Transfer Accumulator to Y Register
    // Function:    Y = A
    // Flags Out:   N, Z
    fn tay(&mut self, _bus: &mut Bus) -> u8 {  
        self.y = self.a;
        self.set_flag(FLAG6502_Z, self.y        == 0x00); 
        self.set_flag(FLAG6502_N, self.y & 0x80 != 0x00); 
        0
    }
    
    // Instruction: Transfer Stack Pointer to X Register
    // Function:    X = stack pointer
    // Flags Out:   N, Z
    fn tsx(&mut self, _bus: &mut Bus) -> u8 { 
        self.x = self.stkp;
        self.set_flag(FLAG6502_Z, self.x        == 0x00); 
        self.set_flag(FLAG6502_N, self.x & 0x80 != 0x00); 
        0
     }
    
    // Instruction: Transfer X Register to Accumulator
    // Function:    A = X
    // Flags Out:   N, Z
    fn txa(&mut self, _bus: &mut Bus) -> u8 { 
        self.a = self.x;
        self.set_flag(FLAG6502_Z, self.a        == 0x00); 
        self.set_flag(FLAG6502_N, self.a & 0x80 != 0x00); 
        0
     }

    
    // Instruction: Transfer X Register to Stack Pointer
    // Function:    stack pointer = X
    fn txs(&mut self, _bus: &mut Bus) -> u8 { 
        self.stkp = self.x;
        0
    }

    
    // Instruction: Transfer Y Register to Accumulator
    // Function:    A = Y
    // Flags Out:   N, Z
    fn tya(&mut self, _bus: &mut Bus) -> u8 {  
        self.a = self.y;
        self.set_flag(FLAG6502_Z, self.a        == 0x00); 
        self.set_flag(FLAG6502_N, self.a & 0x80 != 0x00); 
        0
    }


}
