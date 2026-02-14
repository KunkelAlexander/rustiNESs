use crate::bus::Bus;

pub const FLAG6502_C: u8 = 1 << 0; // Carry Bit
pub const FLAG6502_Z: u8 = 1 << 1; // Zero
pub const FLAG6502_I: u8 = 1 << 2; // Disable Interrupts
pub const FLAG6502_D: u8 = 1 << 3; // Decimal Mode (unused in this implementation)
pub const FLAG6502_B: u8 = 1 << 4; // Break
pub const FLAG6502_U: u8 = 1 << 5; // Unused
pub const FLAG6502_V: u8 = 1 << 6; // Overflow
pub const FLAG6502_N: u8 = 1 << 7; // Negative


type OpFn = fn(&mut Olc6502, &mut Bus) -> u8;
type AddrFn = fn(&mut Olc6502, &mut Bus) -> u8;

#[derive(Copy, Clone)]
pub struct Instruction {
    pub name: &'static str,
    pub addrmode: AddrFn,
    pub operate: OpFn,
    pub cycles: u8,
}


    

pub struct Olc6502 {
    // registers
    a      : u8,  // Accumulator register
    x      : u8,   // X register
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

    
    lookup: [Instruction; 256],
}




impl Olc6502 {
    pub fn new() -> Self {
        Self {
            // init registers etc
            a: 0,
            x: 0,
            y: 0,
            stkp: 0,
            pc: 0,
            status: 0,

            fetched: 0, 
            addr_abs: 0, 
            addr_rel: 0, 
            opcode: 0,
            cycles: 0,

            lookup: Self::build_lookup(),
        }
    }

    pub fn read(&self, bus: &Bus, addr: u16) -> u8 {
        let read_only: bool = false;
        bus.read(addr, read_only)
    }

    pub fn write(&self, bus: &mut Bus, addr: u16, data: u8) {
        bus.write(addr, data)
    }

    pub fn get_flag(&self, f: u8) -> u8 {
        if (self.status & f) != 0 { 1 } else { 0 }
    }

    pub fn set_flag(&mut self, f: u8, v: bool) {
        if v {
            self.status |= f;
        } else {
            self.status &= !f;
        }
    }



    fn xxx(&mut self, _bus: &mut Bus) -> u8 { 0 }

    // add data fetched from memory to accumulator, including the carry bit
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

    fn and(&mut self, bus: &mut Bus) -> u8 { 
        self.fetch(bus);
        self.a = self.a & self.fetched; 
        self.set_flag(FLAG6502_Z, self.a == 0x00);
        self.set_flag(FLAG6502_N, self.a &  0x80 != 0);
        1
     }
    fn asl(&mut self, _bus: &mut Bus) -> u8 { 0 }

     // helper function to implement branching
     // Consumes 1 or 2 cycles and updates pc to pc + addr_rel
    fn branch(&mut self, bus: &mut Bus) {
        self.cycles += 1;
        self.addr_abs = self.pc + self.addr_rel; 

        // second cycle of clock penalty if we cross a page boundary
        if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
            self.cycles += 1; 
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
    fn bit(&mut self, _bus: &mut Bus) -> u8 { 0 }
    
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
    fn brk(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn bvc(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn bvs(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn clc(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(FLAG6502_C, false);
        0
     }
    fn cld(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(FLAG6502_D, false);
        0
     }
    fn cli(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(FLAG6502_I, false);
        0
     }
    fn clv(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(FLAG6502_V, false);
        0
     }
    fn cmp(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn cpx(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn cpy(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn dec(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn dex(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn dey(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn eor(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn inc(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn inx(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn iny(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn jmp(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn jsr(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn lda(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn ldx(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn ldy(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn lsr(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn nop(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn ora(&mut self, _bus: &mut Bus) -> u8 { 0 }
    // Push accumulator to the stack
    // 0x0100 is hard-coded stack-location
    fn pha(&mut self, bus: &mut Bus) -> u8 {
        self.write(bus, (0x0100 as u16) + (self.stkp as u16), self.a);
        self.stkp -= 1;
        0
    }
    fn php(&mut self, _bus: &mut Bus) -> u8 { 0 }
    // Pop from stack
    // 0x0100 is hard-coded stack-location
    fn pla(&mut self, bus: &mut Bus) -> u8 { 
        self.stkp += 1; 
        self.a = self.read(bus, (0x0100 as u16) + (self.stkp as u16));
        self.set_flag(FLAG6502_Z, self.a == 0x00); 
        self.set_flag(FLAG6502_N, self.a & 0x80 != 0); 
        0
    }
    fn plp(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn rol(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn ror(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn rti(&mut self, bus: &mut Bus) -> u8 { 
        self.stkp += 1; 
        self.status = self.read(bus, (0x0100 as u16) + (self.stkp as u16));
        self.status &= !FLAG6502_B;
        self.status &= !FLAG6502_U;
        self.stkp += 1; 
        
        self.pc    = self.read(bus, (0x0100 as u16) + (self.stkp as u16)) as u16;
        self.stkp += 1; 
        self.pc   |= (self.read(bus, (0x0100 as u16) + (self.stkp as u16)) as u16) << 8;
        self.stkp += 1; 
        0
    }
    fn rts(&mut self, _bus: &mut Bus) -> u8 { 0 }
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
    fn sec(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn sed(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn sei(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn sta(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn stx(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn sty(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn tax(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn tay(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn tsx(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn txa(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn txs(&mut self, _bus: &mut Bus) -> u8 { 0 }
    fn tya(&mut self, _bus: &mut Bus) -> u8 { 0 }

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

    // Implied mode
    fn imp(&mut self, _bus: &mut Bus) -> u8 { 
        self.fetched = self.a; 
        0
    }
    // Immediate mode
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
        self.addr_abs &= 0x00FF; // Clear upper bits if addr_abs is not within range
        0
    }
     // Zero-page offset with y-register addressing
    fn zpy(&mut self, bus: &mut Bus) -> u8 { 
        self.addr_abs  = self.read(bus, self.pc) as u16;
        self.addr_abs  = self.addr_abs.wrapping_add(self.y as u16);
        self.pc        = self.pc.wrapping_add(1);
        self.addr_abs &= 0x00FF; // Clear upper
        0
    }
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

        self.addr_abs  = (hi << 8) | lo; 
        
        0
    }
    fn izy(&mut self, bus: &mut Bus) -> u8 { 
        let t : u16    = self.read(bus, self.pc) as u16;
        self.pc        = self.pc.wrapping_add(1);
        let lo : u16   = self.read(bus, (t    ) & 0x00FF) as u16;
        let hi : u16   = self.read(bus, (t + 1) & 0x00FF) as u16; 

        self.addr_abs  = (hi << 8) | lo; 
        self.addr_abs += self.y as u16; 

        if (self.addr_abs & 0xFF00) != (hi << 8) {
            1
        } else {
            0
        }
     }


    // configure CPU into a known state
    pub fn reset(&mut self, bus: &mut Bus) {
        self.a  = 0; 
        self.x  = 0;
        self.y  = 0; 
        self.stkp = 0xFD; 
        self.status = 0x00 | FLAG6502_U;

        self.addr_abs = 0xFFFC;
        let lo: u16 = self.read(bus,self.addr_abs + 0) as u16;
        let hi: u16 = self.read(bus,self.addr_abs + 1) as u16;
        self.addr_rel = 0x0000;
        self.addr_abs = 0x0000;
        self.fetched  = 0x00;
        
        self.cycles = 8;

    }

    pub fn clock(&mut self, bus: &mut Bus) {
        if self.cycles == 0 {
            // Read one byte from bus containing the opcode
            let opcode = bus.read(self.pc, true);
            
            self.pc = self.pc.wrapping_add(1);
            println!("PC={:04X} opcode={:02X}", self.pc, opcode);

            let inst = self.lookup[self.opcode as usize];
            self.cycles = inst.cycles;
            let additional_cycle1 = (inst.addrmode)(self, bus);
            let additional_cycle2 = (inst.operate)(self, bus);

            self.cycles += additional_cycle1 & additional_cycle2;

        }

        self.cycles -= 1;
        
    }

    pub fn irq(&mut self, bus: &mut Bus) {
        if self.get_flag(FLAG6502_I) != 0 {
            self.nmi(bus);
        }
    }

    pub fn nmi(&mut self, bus: &mut Bus) {
        self.write(bus, 0x0100 + self.stkp as u16, ((self.pc >> 8) & 0x00FF) as u8);
        self.stkp -= 1; 
        self.write(bus, 0x0100 + self.stkp as u16, ((self.pc     ) & 0x00FF) as u8);
        self.stkp -= 1; 

        self.set_flag(FLAG6502_B, false);
        self.set_flag(FLAG6502_U, true);
        self.set_flag(FLAG6502_I, true);

        self.write(bus, 0x0100 + self.stkp as u16, self.status);
        self.stkp -= 1; 

        self.addr_abs = 0xFFFE;
        let lo: u16 = self.read(bus,self.addr_abs + 0) as u16;
        let hi: u16 = self.read(bus,self.addr_abs + 1) as u16;
        self.pc = (hi << 8) | lo; 
        
        self.cycles = 7;
    }


    pub fn fetch(&mut self, bus: &mut Bus) -> u8 {
        let inst = self.lookup[self.opcode as usize];

        if inst.addrmode != Olc6502::imp {
            self.fetched = self.read(bus, self.addr_abs);
        }

        self.fetched
    }



    fn build_lookup() -> [Instruction; 256] {
        // default all opcodes to illegal/unknown
        let xxx = Instruction { name: "???", addrmode: Olc6502::imp, operate: Olc6502::xxx, cycles: 2 };
        let mut t = [xxx; 256];

        // Helper macro to make the table readable
        macro_rules! op {
            ($code:expr, $name:expr, $addr:ident, $op:ident, $cy:expr) => {
                t[$code] = Instruction {
                    name: $name,
                    addrmode: Olc6502::$addr,
                    operate: Olc6502::$op,
                    cycles: $cy,
                };
            };
        }

        // ----- Official NMOS 6502 opcodes (base cycles) -----
        op!(0x00, "BRK", imm, brk, 7);
        op!(0x01, "ORA", izx, ora, 6);
        op!(0x05, "ORA", zp0, ora, 3);
        op!(0x06, "ASL", zp0, asl, 5);
        op!(0x08, "PHP", imp, php, 3);
        op!(0x09, "ORA", imm, ora, 2);
        op!(0x0A, "ASL", imp, asl, 2);
        op!(0x0D, "ORA", abs, ora, 4);
        op!(0x0E, "ASL", abs, asl, 6);

        op!(0x10, "BPL", rel, bpl, 2);
        op!(0x11, "ORA", izy, ora, 5);
        op!(0x15, "ORA", zpx, ora, 4);
        op!(0x16, "ASL", zpx, asl, 6);
        op!(0x18, "CLC", imp, clc, 2);
        op!(0x19, "ORA", aby, ora, 4);
        op!(0x1D, "ORA", abx, ora, 4);
        op!(0x1E, "ASL", abx, asl, 7);

        op!(0x20, "JSR", abs, jsr, 6);
        op!(0x21, "AND", izx, and, 6);
        op!(0x24, "BIT", zp0, bit, 3);
        op!(0x25, "AND", zp0, and, 3);
        op!(0x26, "ROL", zp0, rol, 5);
        op!(0x28, "PLP", imp, plp, 4);
        op!(0x29, "AND", imm, and, 2);
        op!(0x2A, "ROL", imp, rol, 2);
        op!(0x2C, "BIT", abs, bit, 4);
        op!(0x2D, "AND", abs, and, 4);
        op!(0x2E, "ROL", abs, rol, 6);

        op!(0x30, "BMI", rel, bmi, 2);
        op!(0x31, "AND", izy, and, 5);
        op!(0x35, "AND", zpx, and, 4);
        op!(0x36, "ROL", zpx, rol, 6);
        op!(0x38, "SEC", imp, sec, 2);
        op!(0x39, "AND", aby, and, 4);
        op!(0x3D, "AND", abx, and, 4);
        op!(0x3E, "ROL", abx, rol, 7);

        op!(0x40, "RTI", imp, rti, 6);
        op!(0x41, "EOR", izx, eor, 6);
        op!(0x45, "EOR", zp0, eor, 3);
        op!(0x46, "LSR", zp0, lsr, 5);
        op!(0x48, "PHA", imp, pha, 3);
        op!(0x49, "EOR", imm, eor, 2);
        op!(0x4A, "LSR", imp, lsr, 2);
        op!(0x4C, "JMP", abs, jmp, 3);
        op!(0x4D, "EOR", abs, eor, 4);
        op!(0x4E, "LSR", abs, lsr, 6);

        op!(0x50, "BVC", rel, bvc, 2);
        op!(0x51, "EOR", izy, eor, 5);
        op!(0x55, "EOR", zpx, eor, 4);
        op!(0x56, "LSR", zpx, lsr, 6);
        op!(0x58, "CLI", imp, cli, 2);
        op!(0x59, "EOR", aby, eor, 4);
        op!(0x5D, "EOR", abx, eor, 4);
        op!(0x5E, "LSR", abx, lsr, 7);

        op!(0x60, "RTS", imp, rts, 6);
        op!(0x61, "ADC", izx, adc, 6);
        op!(0x65, "ADC", zp0, adc, 3);
        op!(0x66, "ROR", zp0, ror, 5);
        op!(0x68, "PLA", imp, pla, 4);
        op!(0x69, "ADC", imm, adc, 2);
        op!(0x6A, "ROR", imp, ror, 2);
        op!(0x6C, "JMP", ind, jmp, 5);
        op!(0x6D, "ADC", abs, adc, 4);
        op!(0x6E, "ROR", abs, ror, 6);

        op!(0x70, "BVS", rel, bvs, 2);
        op!(0x71, "ADC", izy, adc, 5);
        op!(0x75, "ADC", zpx, adc, 4);
        op!(0x76, "ROR", zpx, ror, 6);
        op!(0x78, "SEI", imp, sei, 2);
        op!(0x79, "ADC", aby, adc, 4);
        op!(0x7D, "ADC", abx, adc, 4);
        op!(0x7E, "ROR", abx, ror, 7);

        op!(0x81, "STA", izx, sta, 6);
        op!(0x84, "STY", zp0, sty, 3);
        op!(0x85, "STA", zp0, sta, 3);
        op!(0x86, "STX", zp0, stx, 3);
        op!(0x88, "DEY", imp, dey, 2);
        op!(0x8A, "TXA", imp, txa, 2);
        op!(0x8C, "STY", abs, sty, 4);
        op!(0x8D, "STA", abs, sta, 4);
        op!(0x8E, "STX", abs, stx, 4);

        op!(0x90, "BCC", rel, bcc, 2);
        op!(0x91, "STA", izy, sta, 6);
        op!(0x94, "STY", zpx, sty, 4);
        op!(0x95, "STA", zpx, sta, 4);
        op!(0x96, "STX", zpy, stx, 4);
        op!(0x98, "TYA", imp, tya, 2);
        op!(0x99, "STA", aby, sta, 5);
        op!(0x9A, "TXS", imp, txs, 2);
        op!(0x9D, "STA", abx, sta, 5);

        op!(0xA0, "LDY", imm, ldy, 2);
        op!(0xA1, "LDA", izx, lda, 6);
        op!(0xA2, "LDX", imm, ldx, 2);
        op!(0xA4, "LDY", zp0, ldy, 3);
        op!(0xA5, "LDA", zp0, lda, 3);
        op!(0xA6, "LDX", zp0, ldx, 3);
        op!(0xA8, "TAY", imp, tay, 2);
        op!(0xA9, "LDA", imm, lda, 2);
        op!(0xAA, "TAX", imp, tax, 2);
        op!(0xAC, "LDY", abs, ldy, 4);
        op!(0xAD, "LDA", abs, lda, 4);
        op!(0xAE, "LDX", abs, ldx, 4);

        op!(0xB0, "BCS", rel, bcs, 2);
        op!(0xB1, "LDA", izy, lda, 5);
        op!(0xB4, "LDY", zpx, ldy, 4);
        op!(0xB5, "LDA", zpx, lda, 4);
        op!(0xB6, "LDX", zpy, ldx, 4);
        op!(0xB8, "CLV", imp, clv, 2);
        op!(0xB9, "LDA", aby, lda, 4);
        op!(0xBA, "TSX", imp, tsx, 2);
        op!(0xBC, "LDY", abx, ldy, 4);
        op!(0xBD, "LDA", abx, lda, 4);
        op!(0xBE, "LDX", aby, ldx, 4);

        op!(0xC0, "CPY", imm, cpy, 2);
        op!(0xC1, "CMP", izx, cmp, 6);
        op!(0xC4, "CPY", zp0, cpy, 3);
        op!(0xC5, "CMP", zp0, cmp, 3);
        op!(0xC6, "DEC", zp0, dec, 5);
        op!(0xC8, "INY", imp, iny, 2);
        op!(0xC9, "CMP", imm, cmp, 2);
        op!(0xCA, "DEX", imp, dex, 2);
        op!(0xCC, "CPY", abs, cpy, 4);
        op!(0xCD, "CMP", abs, cmp, 4);
        op!(0xCE, "DEC", abs, dec, 6);

        op!(0xD0, "BNE", rel, bne, 2);
        op!(0xD1, "CMP", izy, cmp, 5);
        op!(0xD5, "CMP", zpx, cmp, 4);
        op!(0xD6, "DEC", zpx, dec, 6);
        op!(0xD8, "CLD", imp, cld, 2);
        op!(0xD9, "CMP", aby, cmp, 4);
        op!(0xDD, "CMP", abx, cmp, 4);
        op!(0xDE, "DEC", abx, dec, 7);

        op!(0xE0, "CPX", imm, cpx, 2);
        op!(0xE1, "SBC", izx, sbc, 6);
        op!(0xE4, "CPX", zp0, cpx, 3);
        op!(0xE5, "SBC", zp0, sbc, 3);
        op!(0xE6, "INC", zp0, inc, 5);
        op!(0xE8, "INX", imp, inx, 2);
        op!(0xE9, "SBC", imm, sbc, 2);
        op!(0xEA, "NOP", imp, nop, 2);
        op!(0xEC, "CPX", abs, cpx, 4);
        op!(0xED, "SBC", abs, sbc, 4);
        op!(0xEE, "INC", abs, inc, 6);

        op!(0xF0, "BEQ", rel, beq, 2);
        op!(0xF1, "SBC", izy, sbc, 5);
        op!(0xF5, "SBC", zpx, sbc, 4);
        op!(0xF6, "INC", zpx, inc, 6);
        op!(0xF8, "SED", imp, sed, 2);
        op!(0xF9, "SBC", aby, sbc, 4);
        op!(0xFD, "SBC", abx, sbc, 4);
        op!(0xFE, "INC", abx, inc, 7);

        t
    }
}
