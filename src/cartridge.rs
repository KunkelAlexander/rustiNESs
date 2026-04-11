use crate::interfaces::{CartridgeInterface, MapperInterface};
use crate::mapper::Mapper000;

// Documentation on cartridge formats
// https://nescartdb.com/

enum MIRROR
{
    Horizontal,
    Vertical,
    OnescreenLo,
    OnescreenHi,
}

// iNES format header
struct INesHeader {
    prg_rom_chunks : u8, 
    chr_rom_chunks : u8, 
    mapper1        : u8, 
    mapper2        : u8, 
    prg_ram_size   : u8, 
    tv_system1     : u8, 
    tv_system2     : u8
}

//Represent NES without cartridge via empty cartridge
pub struct EmptyCartridge;

impl CartridgeInterface for EmptyCartridge {
    fn read_cpu(&mut self, addr: u16) -> Option<u8>             {None}
    fn write_cpu(&mut self, addr: u16, data: u8) -> Option<()>  {None}
    fn read_ppu(& self, addr: u16) -> Option<u8>                {None}
    fn write_ppu(&mut self, addr: u16, data: u8) -> Option<()>  {None}
    fn map_nametable_addr(&self, addr: u16) -> u16              {0}
}




pub struct Cartridge {
    v_prg_memory: Vec<u8>,
    v_chr_memory: Vec<u8>,
    n_mapper_id:  u8,                       // which mapper are we using?
    n_prg_banks:  u8,                       // how many banks of prg memory? 
    n_chr_banks:  u8,                       // how many banks of chr memory?
    mirror:       MIRROR,
    mapper:       Box<dyn MapperInterface>, // Reference to mapper
}

impl Cartridge {

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
		let n_mapper_id: u8 = ((header.mapper2 >> 4) << 4) | (header.mapper1 >> 4);
		let mirror: MIRROR  = if (header.mapper1 & 0x01) != 0 {MIRROR::Vertical} else {MIRROR::Horizontal};

        let mut offset: usize = 16; 
        
		// If a "trainer" exists we just need to read past
		// it before we get to the good stuff
		if header.mapper1 & 0x04 != 0 {
			offset += 512;
        }


	    // "Discover" File Format
		let n_file_type : u8 = 1;
        let prg_memory: Vec<u8>;
        let chr_memory: Vec<u8>;

		if (n_file_type == 1)
		{
            let prg_size = (header.prg_rom_chunks as usize) * 16384;
            let chr_size = (header.chr_rom_chunks as usize) *  8192;

            if data.len() < offset + prg_size + chr_size {
                return Err("File truncated".into());
            }

            prg_memory = data[offset..offset+prg_size].to_vec();
            offset += prg_size;
            
            chr_memory = if header.chr_rom_chunks == 0 {
                vec![0; 8192] // CHR RAM
            } else {
                data[offset..offset + chr_size].to_vec()
            };
		} else  {
            return Err("Unsupported file type".into());
		}

		// Load appropriate mapper
		let mapper: Box<dyn MapperInterface> = match n_mapper_id {
		 0 => Box::new(Mapper000 { prg_banks: header.prg_rom_chunks, chr_banks: header.chr_rom_chunks }),
         _ => return Err("Unsupported mapper".into()),
		};

        Ok(Self {
            v_prg_memory: prg_memory,
            v_chr_memory: chr_memory,
            n_mapper_id:  n_mapper_id,
            n_prg_banks:  header.prg_rom_chunks,
            n_chr_banks:  header.chr_rom_chunks,
            mirror,
            mapper
        })

    }
}


// Rust has cool functional elements
// map captures the Option returned by read and write functions as mapped_addr, returns it if it is None, else it applies it to the Lambda function
impl CartridgeInterface for Cartridge {
    fn read_cpu(&mut self, addr: u16) -> Option<u8> {
        self.mapper.cpu_map_read( addr      ).map(|mapped_addr|  self.v_prg_memory[mapped_addr])
    }
    fn write_cpu(&mut self, addr: u16, data: u8) -> Option<()> {
        self.mapper.cpu_map_write(addr, data).map(|mapped_addr| {self.v_prg_memory[mapped_addr] = data;})
    }
    fn read_ppu(&    self, addr: u16) -> Option<u8> {
        self.mapper.ppu_map_read( addr      ).map(|mapped_addr|  self.v_chr_memory[mapped_addr])
    }
    fn write_ppu(&mut self, addr: u16, data: u8) -> Option<()> {
        self.mapper.ppu_map_write(addr, data).map(|mapped_addr| {self.v_chr_memory[mapped_addr] = data;})
    }

    // This takes an input address in the range 0x2000 to 0x3EFF and maps it to the appropriate address in the name table based
    // on the mirroring mode
    // I represent the two nametables as a 2*1024 byte array - we therefore need to offset by 1024 = 0x0400 to get to the second nametable (page 1)
    fn map_nametable_addr(&self, addr: u16) -> u16 {
        let offset = addr & 0x0FFF;
        match self.mirror {
            MIRROR::Vertical => match offset {
                0x0000..=0x03FF =>         offset & 0x03FF,    // NT0 -> page 0
                0x0400..=0x07FF => 1024 + (offset & 0x03FF),   // NT1 -> page 1
                0x0800..=0x0BFF =>         offset & 0x03FF,    // NT2 -> page 0
                0x0C00..=0x0FFF => 1024 + (offset & 0x03FF),   // NT3 -> page 1
                _ => unreachable!(),
            },
            MIRROR::Horizontal => match offset {
                0x0000..=0x03FF =>         offset & 0x03FF,    // NT0 -> page 0
                0x0400..=0x07FF =>         offset & 0x03FF,    // NT1 -> page 0
                0x0800..=0x0BFF => 1024 + (offset & 0x03FF),   // NT2 -> page 1
                0x0C00..=0x0FFF => 1024 + (offset & 0x03FF),   // NT3 -> page 1
                _ => unreachable!(),
            },
            MIRROR::OnescreenLo =>         offset & 0x03FF,    // all -> page 0
            MIRROR::OnescreenHi => 1024 + (offset & 0x03FF),   // all -> page 1
        }
    }
}



