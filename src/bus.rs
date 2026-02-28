use crate::interfaces::{CartridgeInterface, BusInterface, PpuInterface};
use crate::ppu::Olc2c02;

// SimpleBus only containing 64 KB of RAM used in 6502 demo
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

impl BusInterface for SimpleBus {
    fn read(&mut self, addr: u16, _read_only: bool) -> u8 {
        if addr >= 0x0000 && addr <= 0xFFFF {
           self.ram[addr as usize]
        } else {
            0
        }
    }
    fn write(&mut self, addr: u16, data: u8) {
        if addr >= 0x0000 && addr <= 0xFFFF {
            self.ram[addr as usize] = data;
        }
        
    }
}


// NES bus containing 2 KB of RAM 
pub struct Bus {
    cpu_ram:              [u8; 2048],
    pub ppu:              Olc2c02,
    cartridge:            Box<dyn CartridgeInterface>
}

impl Bus {
    pub fn new(
        cartridge: Box<dyn CartridgeInterface>,
    ) -> Self {
        Self {
            cpu_ram: [0; 2048],
            ppu: Olc2c02::new(),
            cartridge: cartridge
        }
    }

    pub fn get_ram(&self, start: u16, len: usize) -> Vec<u8> {
        let start = start as usize;
        let end = start + len;

        self.cpu_ram[start..end].to_vec()
    }

    pub fn reset(&mut self) {
        self.cpu_ram              = [0u8; 2048];
    }

    pub fn insert_cartridge(&mut self, cartridge: Box<dyn CartridgeInterface>) {
        self.cartridge = cartridge;
    }
}

impl BusInterface for Bus {
    fn read(&mut self, addr: u16, read_only: bool) -> u8 {
        // Cartridge gets first chance
        if let Some(data) = self.cartridge.read_cpu(addr) {
            return data;
        }
        // System RAM (mirrored every 2 KB)
        if (addr >= 0x0000 && addr <= 0x1FFF)
        {
           return self.cpu_ram[(addr & 0x07FF) as usize];
        }
        // PPU Address range, mirrored every 8 bytes
        if (addr >= 0x2000 && addr <= 0x3FFF)
        {
            return self.ppu.read_cpu(addr & 0x0007, read_only);
        }
        0
    }
    fn write(&mut self, addr: u16, data: u8) {

        // Cartridge gets first chance
        if self.cartridge.write_cpu(addr, data).is_some() {
        }
        // System RAM (mirrored every 2 KB)
        else if (addr >= 0x0000 && addr <= 0x1FFF)
        {
           self.cpu_ram[(addr & 0x07FF) as usize] = data;
        }
        // PPU Address range, mirrored every 8 bytes
        if (addr >= 0x2000 && addr <= 0x3FFF)
        {
            self.ppu.write_cpu(addr & 0x0007, data);
        }
        
    }
}


