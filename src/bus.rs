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
    cartridge:            Box<dyn CartridgeInterface>,
    pub controller:       [u8; 2], // this needs to be set externally
    controller_state:     [u8; 2], // store snapshots of the inputs when the corresponding memory address is written to. 
}

impl Bus {
    pub fn new(
        cartridge: Box<dyn CartridgeInterface>,
    ) -> Self {
        Self {
            cpu_ram: [0; 2048],
            ppu: Olc2c02::new(),
            cartridge: cartridge,
            controller: [0; 2],
            controller_state: [0; 2]
        }
    }

    pub fn get_ram(&self, start: u16, len: usize) -> Vec<u8> {
        let start = start as usize;
        let end = start + len;

        self.cpu_ram[start..end].to_vec()
    }

    
    pub fn get_pattern_table(&self, i: u8, palette: u8) -> Vec<u8> {
        self.ppu.get_pattern_table(i, palette, self.cartridge.as_ref())
    }

    pub fn get_name_table(&self) -> Vec<u8> {
        self.ppu.get_name_table()
    }
    // Does not set RAM to zero
    pub fn reset(&mut self) {
    }

    pub fn clock(&mut self) {
        self.ppu.clock(self.cartridge.as_mut());
    }

    pub fn insert_cartridge(&mut self, cartridge: Box<dyn CartridgeInterface>) {
        self.cartridge = cartridge;
    }

    
    pub fn set_controller(&mut self, i: usize, x: u8, z: u8, a: u8, s: u8, up: u8, down: u8, left: u8, right: u8) {
        self.controller[i]  = 
          x     * (1 << 7) 
        + z     * (1 << 6) 
        + a     * (1 << 5) 
        + s     * (1 << 4) 
        + up    * (1 << 3) 
        + down  * (1 << 2) 
        + left  * (1 << 1) 
        + right * (1 << 0);
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
            return self.ppu.read_cpu(addr & 0x0007, read_only, self.cartridge.as_mut());
        }
        // Read most significant bit of controller state via pop
        else if (addr >= 0x4016 && addr <= 0x4017)
        {
            let temp = ((self.controller_state[(addr & 0x0001) as usize] & 0x80) > 0) as u8;
            self.controller_state[(addr & 0x0001) as usize] <<= 1; 
            return temp;
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
        else if (addr >= 0x2000 && addr <= 0x3FFF)
        {
            self.ppu.write_cpu(addr & 0x0007, data, self.cartridge.as_mut());
        }
        // Copy external controller state into internal register
        else if (addr >= 0x4016 && addr <= 0x4017)
        {
            self.controller_state[(addr & 0x0001) as usize] = self.controller[(addr & 0x0001) as usize];
        }
        
    }
}


