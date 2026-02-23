pub struct Olc2c02 {
    name_table: [u8; 2*1024], // physical VRAM for the name tables
    palette:    [u8; 32],     // physical VRAM for the palletes
    pattern:    [u8; 2*4096],
}

impl Olc2c02 {

    pub fn new() -> Self {
        Self {
            name_table: [0u8; 2*1024], 
            palette:    [0u8; 32],
            pattern:    [0u8; 2*4096],
        }
    }

    pub fn read_cpu(&self, addr: u16, _read_only: bool) -> u8 {
        if addr >= 0x0000 && addr <= 0xFFFF {
           self.ram[addr as usize]
        } else {
            0
        }
    }
    pub fn write_cpu(&mut self, addr: u16, data: u8) {
        if addr >= 0x0000 && addr <= 0xFFFF {
            self.ram[addr as usize] = data;
        }
        
    }

    
    pub fn read_ppu(&self, addr: u16, _read_only: bool) -> u8 {
        if addr >= 0x0000 && addr <= 0xFFFF {
           self.ram[addr as usize]
        } else {
            0
        }
    }
    pub fn write_ppu(&mut self, addr: u16, data: u8) {
        if addr >= 0x0000 && addr <= 0xFFFF {
            self.ram[addr as usize] = data;
        }
        
    }
}



