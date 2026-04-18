use crate::cartridge::Cartridge;

pub trait BusInterface { 
    fn read (&mut self, addr: u16, _read_only: bool) -> u8; 
    fn write(&mut self, addr: u16, data: u8); 
}


pub trait PpuInterface { 
    fn read_cpu (&mut self, addr: u16, _read_only: bool, cartridge: &mut dyn CartridgeInterface) -> u8; 
    fn write_cpu(&mut self, addr: u16, data: u8,         cartridge: &mut dyn CartridgeInterface); 
    fn read_ppu (&    self, addr: u16,                   cartridge: &    dyn CartridgeInterface) -> Option<u8>; 
    fn write_ppu(&mut self, addr: u16, data: u8,         cartridge: &mut dyn CartridgeInterface); 
}


// Option return values indicate write and read success 
pub trait CartridgeInterface { 
    fn read_cpu (&mut self, addr: u16          ) -> Option<u8>; 
    fn write_cpu(&mut self, addr: u16, data: u8) -> Option<()>; 
    fn read_ppu (&    self, addr: u16          ) -> Option<u8>; 
    fn write_ppu(&mut self, addr: u16, data: u8) -> Option<()>; 
    fn map_nametable_addr(&self, addr: u16)      -> u16;
    fn reset(&mut self);
}

pub trait MapperInterface {
    fn cpu_map_read (&    self, addr: u16          ) -> Option<usize>;
    fn cpu_map_write(&mut self, addr: u16, data: u8) -> Option<usize>;
    fn ppu_map_read (&    self, addr: u16          ) -> Option<usize>;
    fn ppu_map_write(&mut self, addr: u16, data: u8) -> Option<usize>;
    fn reset(&mut self);
}