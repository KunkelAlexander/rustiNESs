use crate::interfaces::{MapperInterface};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Mapper {
    Mapper000(Mapper000),
}

impl MapperInterface for Mapper {
    fn cpu_map_read(&self, addr: u16) -> Option<usize> {
        match self { Mapper::Mapper000(m) => m.cpu_map_read(addr) }
    }
    fn cpu_map_write(&mut self, addr: u16, data: u8) -> Option<usize> {
        match self { Mapper::Mapper000(m) => m.cpu_map_write(addr, data) }
    }
    fn ppu_map_read(&self, addr: u16) -> Option<usize> {
        match self { Mapper::Mapper000(m) => m.ppu_map_read(addr) }
    }
    fn ppu_map_write(&mut self, addr: u16, data: u8) -> Option<usize> {
        match self { Mapper::Mapper000(m) => m.ppu_map_write(addr, data) }
    }
    fn reset(&mut self) {
        match self { Mapper::Mapper000(m) => m.reset() }
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
    fn cpu_map_read(&self, addr: u16) -> Option<usize> {
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
    fn ppu_map_read (&self, addr: u16) -> Option<usize> {
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