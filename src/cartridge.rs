// https://nescartdb.com/

pub struct Cartridge {
    v_prg_memory: Vec<u8>,
    v_chr_memory: Vec<u8>,
    n_mapper_id:  u8, // which mapper are we using?
    n_prg_banks:  u8, // how many banks of prg memory? 
    n_chr_banks:  u8  // how many banks of chr memory?
}

impl Cartridge {

    // iNES format header
    struct SHeader {
        name           : str, 
        prg_rom_chunks : u8, 
        chr_rom_chunks : u8, 
        mapper1        : u8, 
        mapper2        : u8, 
        prg_ram_size   : u8, 
        tv_system1     : u8, 
        tv_system2     : u8, 
        unused         : str
    }

    pub fn new(filename: &str) -> Self {
        Self {
            v_prg_memory: Vec<u8>::new(),
            v_chr_memory: Vec<u8>::new(),
        }
    }

    // Read 
    pub fn read_cpu(&self, addr: u16, _read_only: bool) -> bool {
        if addr >= 0x0000 && addr <= 0xFFFF {
           self.ram[addr as usize]
        } else {
            0
        }
    }
    pub fn write_cpu(&mut self, addr: u16, data: u8) -> bool{
        if addr >= 0x0000 && addr <= 0xFFFF {
            self.ram[addr as usize] = data;
        }
        
    }

    
    pub fn read_ppu(&self, addr: u16, _read_only: bool) -> bool {
        if addr >= 0x0000 && addr <= 0xFFFF {
           self.ram[addr as usize]
        } else {
            0
        }
    }
    pub fn write_ppu(&mut self, addr: u16, data: u8) -> bool {
        if addr >= 0x0000 && addr <= 0xFFFF {
            self.ram[addr as usize] = data;
        }
        
    }
}



