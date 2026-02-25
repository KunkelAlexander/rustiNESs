use crate::interfaces::CpuBus;

pub struct SimpleBus {
    ram: [u8; 1024*64],
}

impl SimpleBus {
    pub fn new() -> Self {
        Self {
            ram: [0; 1024 * 64],
        }
    }

    pub fn get_ram(&self, start: u16, len: usize) -> Vec<u8> {
        let start = start as usize;
        let end = start + len;

        self.ram[start..end].to_vec()
    }

    pub fn reset(&mut self) {
        self.ram = [0u8; 1024*64];
    }
}

impl CpuBus for SimpleBus {
    fn read_cpu(&mut self, addr: u16, _read_only: bool) -> u8 {
        if addr >= 0x0000 && addr <= 0xFFFF {
           self.ram[addr as usize]
        } else {
            0
        }
    }
    fn write_cpu(&mut self, addr: u16, data: u8) {
        if addr >= 0x0000 && addr <= 0xFFFF {
            self.ram[addr as usize] = data;
        }
        
    }
}



