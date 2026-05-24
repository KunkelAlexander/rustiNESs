pub trait BusInterface { 
    fn read (&mut self, addr: u16, _read_only: bool) -> u8; 
    fn write(&mut self, addr: u16, data: u8); 
}


pub trait PpuInterface<C: CartridgeInterface> { 
    fn read_cpu (&mut self, addr: u16, _read_only: bool, cartridge: &mut C) -> u8; 
    fn write_cpu(&mut self, addr: u16, data: u8,         cartridge: &mut C); 
    fn read_ppu (&mut self, addr: u16,                   cartridge: &mut C) -> Option<u8>; 
    fn write_ppu(&mut self, addr: u16, data: u8,         cartridge: &mut C); 
}


// Option return values indicate write and read success 
pub trait CartridgeInterface { 
    fn read_cpu (&mut self, addr: u16          ) -> Option<u8>; 
    fn write_cpu(&mut self, addr: u16, data: u8) -> Option<()>; 
    fn read_ppu (&mut self, addr: u16          ) -> Option<u8>; 
    fn write_ppu(&mut self, addr: u16, data: u8) -> Option<()>; 
    fn map_nametable_addr(&self, addr: u16)      -> u16;
    fn reset(&mut self);
}

pub trait MapperInterface {
    fn cpu_map_read (&mut self, addr: u16          ) -> Option<usize>;
    fn cpu_map_write(&mut self, addr: u16, data: u8) -> Option<usize>;
    fn ppu_map_read (&mut self, addr: u16          ) -> Option<usize>;
    fn ppu_map_write(&mut self, addr: u16, data: u8) -> Option<usize>;
    // This is required for mapper 163 to pass the protection
    fn cpu_read     (&self, _addr: u16)              -> Option<u8> {None}
    fn reset(&mut self);
}

pub trait ApuInterface { 
    fn read_cpu (&mut self, addr: u16,         ) -> u8; 
    fn write_cpu(&mut self, addr: u16, data: u8);
}