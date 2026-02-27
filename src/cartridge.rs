use crate::interfaces::{CartridgeInterface};

// https://nescartdb.com/

enum MIRROR
{
    Horizontal,
    Vertical,
    OnescreenLo,
    OnescreenHi,
}

pub struct Cartridge {
    v_prg_memory: Vec<u8>,
    v_chr_memory: Vec<u8>,
    n_mapper_id:  u8, // which mapper are we using?
    n_prg_banks:  u8, // how many banks of prg memory? 
    n_chr_banks:  u8, // how many banks of chr memory?
    mirror:       MIRROR,
}

impl Cartridge {

    // iNES format header
    struct INesHeader {
        name           : str, 
        prg_rom_chunks : u8, 
        chr_rom_chunks : u8, 
        mapper1        : u8, 
        mapper2        : u8, 
        prg_ram_size   : u8, 
        tv_system1     : u8, 
        tv_system2     : u8, 
        unused         : str,
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 16 {
            return Err("File too small".into())
        }
        
        // Validate NES magic number
        if &data[0..4] != b"NES\x1A" {
            return Err("Not a valid iNES file".into());
        }


        let header = INesHeader {
            prg_rom_chunks: data[4],
            chr_rom_chunks: data[5],
            mapper1: data[6],
            mapper2: data[7],
            prg_ram_size: data[8],
            tv_system1: data[9],
            tv_system2: data[10],
        };

		// Determine Mapper ID
		self.n_mapper_id = ((header.mapper2 >> 4) << 4) | (header.mapper1 >> 4);
		let mirror       = if (header.mapper1 & 0x01) != 0 {MIRROR::Vertical} else {MIRROR::Horizontal};

        let mut offset = 16; 
        
		// If a "trainer" exists we just need to read past
		// it before we get to the good stuff
		if header.mapper1 & 0x04 != 0 {
			offset += 512;
        }


	    // "Discover" File Format
		let n_file_type : u8 = 1;

		if (nFileType == 0)
		{

		}

		if (nFileType == 1)
		{
            let prg_size = (header.prg_rom_chunks as usize) * 16384;
            let chr_size = (header.chr_rom_chunks as usize) *  8192;

            if data.len() < offset + prg_size + chr_size {
                return Err("File truncated".into());
            }

            let prg_memory = data[offset..offset+prg_size].to_vec();
            offset += prg_size;
            let chr_memory = data[offset..offset+chr_size].to_vec();
		}

		if (nFileType == 2)
		{

		}

		// Load appropriate mapper
		match self.n_mapper_id
		{
		 0 => pMapper = std::make_shared<Mapper_000>(header.prg_rom_chunks, header.chr_rom_chunks); break;
		}

        Ok(Self {
            v_prg_memory: prg_memory,
            v_chr_memory: chr_memory,
            n_mapper_id: mapper_id,
            n_prg_banks: header.prg_rom_chunks,
            n_chr_banks: header.chr_rom_chunks,
            mirror,
        })

    }

impl CartridgeInterface for Cartridge {
    // Read 
    pub fn read_cpu(&self, addr: u16) -> bool {
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
        let mapped_addr : usize = 0;

        if self.mapper.ppu_map_read(addr, mapped_addr) {
            data = self.v_chr_memory[mapped_adr]; 
        }
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



