
pub trait BusInterface { 
    fn read(&mut self, addr: u16, _read_only: bool) -> u8; 
    fn write(&mut self, addr: u16, data: u8); 
}


pub trait PpuInterface { 
    fn read_cpu(&mut self, addr: u16, _read_only: bool) -> u8; 
    fn write_cpu(&mut self, addr: u16, data: u8) -> bool; 
    fn read_ppu(&mut self, addr: u16, _read_only: bool, cartridge: &mut dyn CartridgeInterface) -> Option<u8>; 
    fn write_ppu(&mut self, addr: u16, data: u8, cartridge: &mut dyn CartridgeInterface) -> bool; 
}


pub trait CartridgeInterface { 
    fn read_cpu(&mut self, addr: u16, _read_only: bool) -> Option<u8>; 
    fn write_cpu(&mut self, addr: u16, data: u8) -> bool; 
}