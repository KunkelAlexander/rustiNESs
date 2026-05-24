use crate::interfaces::{MapperInterface};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Mapper {
    Mapper000(Mapper000),
    Mapper163(Mapper163),
}

impl MapperInterface for Mapper {
    fn cpu_read(&self, addr: u16) -> Option<u8> {
        match self {
            Mapper::Mapper000(m)     => m.cpu_read(addr),
            Mapper::Mapper163(m)     => m.cpu_read(addr),
        }
    }

    fn cpu_map_read(&mut self, addr: u16) -> Option<usize> {
        match self { 
            Mapper::Mapper000(m) => m.cpu_map_read(addr),
            Mapper::Mapper163(m) => m.cpu_map_read(addr),
        }
    }
    fn cpu_map_write(&mut self, addr: u16, data: u8) -> Option<usize> {
        match self { 
            Mapper::Mapper000(m) => m.cpu_map_write(addr, data),
            Mapper::Mapper163(m) => m.cpu_map_write(addr, data),
        }
    }
    fn ppu_map_read(&mut self, addr: u16) -> Option<usize> {
        match self { 
            Mapper::Mapper000(m) => m.ppu_map_read(addr),
            Mapper::Mapper163(m) => m.ppu_map_read(addr),
        }
    }
    fn ppu_map_write(&mut self, addr: u16, data: u8) -> Option<usize> {
        match self { 
            Mapper::Mapper000(m) => m.ppu_map_write(addr, data),
            Mapper::Mapper163(m) => m.ppu_map_write(addr, data),
        }
    }
    fn reset(&mut self) {
        match self { 
            Mapper::Mapper000(m) => m.reset(), 
            Mapper::Mapper163(m) => m.reset(), 
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Mapper000 {
    pub prg_banks: u8,
    pub chr_banks: u8,
}

impl MapperInterface for Mapper000 {
   
	// if PRGROM is 16KB
	//     CPU Address Bus          PRG ROM
	//     0x8000 -> 0xBFFF: Map    0x0000 -> 0x3FFF
	//     0xC000 -> 0xFFFF: Mirror 0x0000 -> 0x3FFF
	// if PRGROM is 32KB
	//     CPU Address Bus          PRG ROM
	//     0x8000 -> 0xFFFF: Map    0x0000 -> 0x7FFF	
    fn cpu_map_read(&mut self, addr: u16) -> Option<usize> {
        if addr >= 0x8000 {
            let mapped: usize = (addr & (if self.prg_banks > 1 {0x7FFF} else {0x3FFF})) as usize;
            Some(mapped)
        } else {
            None
        }
    }

    fn cpu_map_write(&mut self, addr: u16, _data: u8) -> Option<usize> {
        if addr >= 0x8000 {
            let mapped: usize = (addr & (if self.prg_banks > 1 {0x7FFF} else {0x3FFF})) as usize;
            Some(mapped)
        } else {
            None
        }
    }

	// There is no mapping required for PPU
	// PPU Address Bus          CHR ROM
	// 0x0000 -> 0x1FFF: Map    0x0000 -> 0x1FFF
    fn ppu_map_read (&mut self, addr: u16) -> Option<usize> {
        if addr <= 0x1FFF {
            Some(addr as usize)
        } else {
            None
        }
    }

    fn ppu_map_write(&mut self, addr: u16, _data: u8) -> Option<usize> {
        if addr <= 0x1FFF && self.chr_banks == 0 {
            Some(addr as usize)
        } else {
            None
        }
    }

    fn reset(&mut self) {
        return;
    }
}

// https://www.nesdev.org/wiki/INES_Mapper_163
// 32 KB PRG-ROM  - game
// 8 KB CHR-RAM   - graphics
// 8 KB PRG-GAM   - savegame
#[derive(Serialize, Deserialize)]
pub struct Mapper163 {
    pub prg_banks: u8,
    pub chr_banks: u8,

    prg_low:  u8,  
    prg_high: u8,    
    mode:     u8,    
    feedback: u8,   
    chr_auto: bool,
    chr_bank: u8,
}

impl Mapper163 {
    pub fn new(prg_banks: u8, chr_banks: u8) -> Self {
        Self {
            prg_banks: prg_banks,
            chr_banks: chr_banks,
            prg_low:   0,
            prg_high:  0,
            mode:      0,
            feedback:  0,
            chr_auto:  false,
            chr_bank:  0,
        }
    }

    fn maybe_swap01(&self, data: u8) -> u8 {
        if self.mode & 0x01 != 0 {
            (data & !0x03) | ((data & 0x01) << 1) | ((data & 0x02) >> 1)
        } else {
            data
        }
    }

    fn prg_bank_32k(&self) -> usize {
        let banks_32k = (self.prg_banks as usize / 2).max(1);

        let mut bank = ((self.prg_high as usize & 0x03) << 4)
            | (self.prg_low as usize & 0x0F);

        if self.mode & 0x04 == 0 {
            bank = (bank & !0x03) | 0x03;
        }

        bank % banks_32k
    }
}

impl MapperInterface for Mapper163 {
    fn cpu_map_read(&mut self, addr: u16) -> Option<usize> {
        match addr {
            0x8000..=0xFFFF => {
                let bank = self.prg_bank_32k();
                Some(bank * 0x8000 + (addr as usize & 0x7FFF))
            }
            _ => None,
        }
    }

    fn cpu_read(&self, addr: u16) -> Option<u8> {
        // $5500/$5501 + mirroring
        if (addr & 0xF300) == 0x5100 {
            Some((self.feedback ^ 1) << 2)
        } else {
            None
        }
    }

    fn cpu_map_write(&mut self, addr: u16, data: u8) -> Option<usize> {
        match addr & 0xFF00 {
            0x5000 => {
                let d = self.maybe_swap01(data);
                self.prg_low = d & 0x0F;
                self.chr_auto = d & 0x80 != 0;
                None
            }
            0x5200 => {
                let d = self.maybe_swap01(data);
                self.prg_high = d & 0x03;
                None
            }
            0x5300 => {
                // Not affected by its own bit-swap.
                self.mode = data;
                None
            }
            0x5100 => {
                if addr & 1 == 0 {
                    let d = self.maybe_swap01(data);
                    self.feedback = (d >> 2) & 1;
                } else {
                    let d = self.maybe_swap01(data);
                    if d & 0x01 != 0 {
                        self.feedback ^= 1;
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn ppu_map_read(&mut self, addr: u16) -> Option<usize> {
        if self.chr_auto && addr >= 0x2000 {
            if (addr & 0x03FF) < 0x03C0 {
                self.chr_bank = ((addr >> 9) & 1) as u8;
            }
            return None;
        }

        if addr <= 0x1FFF {
            let mapped = if self.chr_auto {
                (self.chr_bank as usize * 0x1000) + (addr as usize & 0x0FFF)
            } else {
                addr as usize
            };
            Some(mapped % 0x2000)
        } else {
            None
        }
    }

    fn ppu_map_write(&mut self, addr: u16, _data: u8) -> Option<usize> {
        if addr <= 0x1FFF && self.chr_banks == 0 {
            Some(addr as usize)
        } else {
            None
        }
    }

    fn reset(&mut self) {
        self.prg_low  = 0;
        self.prg_high = 0;
        self.mode     = 0;
        self.feedback = 0;
        self.chr_auto = false;
        self.chr_bank = 0;
    }
}