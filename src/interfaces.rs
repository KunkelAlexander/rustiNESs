
pub trait CpuBus { 
    fn read_cpu(&mut self, addr: u16, _read_only: bool) -> u8; 
    fn write_cpu(&mut self, addr: u16, data: u8); 
}