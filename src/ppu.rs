use crate::{cartridge, interfaces::{CartridgeInterface, PpuInterface}};

pub const SCREEN_W: usize = 256;
pub const SCREEN_H: usize = 240;

/*
This, I handle in my web interface
I feel like it is cleaner if we assign the palette elsewhere for now
#[derive(Clone, Copy, Default)]
pub struct Colour {
    r: u8, 
    g: u8, 
    b: u8
}

pub const NES_PALETTE: [Colour; 64] = [
    Colour { r: 84 , g: 84,  b: 84 },
	Colour { r: 0  , g: 30,  b: 116},
	Colour { r: 8  , g: 16,  b: 144},
	Colour { r: 48 , g: 0,   b: 136},
	Colour { r: 68 , g: 0,   b: 100},
	Colour { r: 92 , g: 0,   b: 48 },
	Colour { r: 84 , g: 4,   b: 0  },
	Colour { r: 60 , g: 24,  b: 0  },
	Colour { r: 32 , g: 42,  b: 0  },
	Colour { r: 8  , g: 58,  b: 0  },
	Colour { r: 0  , g: 64,  b: 0  },
	Colour { r: 0  , g: 60,  b: 0  },
	Colour { r: 0  , g: 50,  b: 60 },
	Colour { r: 0  , g: 0,   b: 0  },
	Colour { r: 0  , g: 0,   b: 0  },
	Colour { r: 0  , g: 0,   b: 0  },
 
	Colour { r: 152, g: 150, b: 152},
	Colour { r: 8  , g: 76,  b: 196},
	Colour { r: 48 , g: 50,  b: 236},
	Colour { r: 92 , g: 30,  b: 228},
	Colour { r: 136, g: 20,  b: 176},
	Colour { r: 160, g: 20,  b: 100},
	Colour { r: 152, g: 34,  b: 32 },
	Colour { r: 120, g: 60,  b: 0  },
	Colour { r: 84 , g: 90,  b: 0  },
	Colour { r: 40 , g: 114, b: 0  },
	Colour { r: 8  , g: 124, b: 0  },
	Colour { r: 0  , g: 118, b: 40 },
	Colour { r: 0  , g: 102, b: 120},
	Colour { r: 0  , g: 0,   b: 0  },
	Colour { r: 0  , g: 0,   b: 0  },
	Colour { r: 0  , g: 0,   b: 0  },

	Colour { r: 236, g: 238, b: 236},
	Colour { r: 76 , g: 154, b: 236},
	Colour { r: 120, g: 124, b: 236},
	Colour { r: 176, g: 98,  b: 236},
	Colour { r: 228, g: 84,  b: 236},
	Colour { r: 236, g: 88,  b: 180},
	Colour { r: 236, g: 106, b: 100},
	Colour { r: 212, g: 136, b: 32 },
	Colour { r: 160, g: 170, b: 0  },
	Colour { r: 116, g: 196, b: 0  },
	Colour { r: 76 , g: 208, b: 32 },
	Colour { r: 56 , g: 204, b: 108},
	Colour { r: 56 , g: 180, b: 204},
	Colour { r: 60 , g: 60,  b: 60 },
	Colour { r: 0  , g: 0,   b: 0  },
	Colour { r: 0  , g: 0,   b: 0  },

	Colour { r: 236, g: 238, b: 236},
	Colour { r: 168, g: 204, b: 236},
	Colour { r: 188, g: 188, b: 236},
	Colour { r: 212, g: 178, b: 236},
	Colour { r: 236, g: 174, b: 236},
	Colour { r: 236, g: 174, b: 212},
	Colour { r: 236, g: 180, b: 176},
	Colour { r: 228, g: 196, b: 144},
	Colour { r: 204, g: 210, b: 120},
	Colour { r: 180, g: 222, b: 120},
	Colour { r: 168, g: 226, b: 144},
	Colour { r: 152, g: 226, b: 180},
	Colour { r: 160, g: 214, b: 228},
	Colour { r: 160, g: 162, b: 160},
	Colour { r: 0  , g: 0  , b: 0  },
	Colour { r: 0  , g: 0  , b: 0  }
];
*/

pub struct Olc2c02 {
    screen:                [u8; SCREEN_H*SCREEN_W],    // Frame buffer
    table_name:            [u8; 2*1024],              // 2 KB of physical VRAM for the name tables
    table_palette:         [u8; 32],                  // 32 Bytes physical VRAM for the palletes
    table_pattern:         [u8; 2*4096],              // 8 KB of physical VRAM for the patterns
    sprite_name_table:     [u8; SCREEN_H*SCREEN_W*2], // Helper for visualisation
    sprite_pattern_table:  [u8; 128*128*2],       // Helper for visualisation
    scanline:       u16, 
    cycle:          u16, 
    pub frame_complete: bool,
    noise_state: u32,
}

impl Olc2c02 {
    pub fn new() -> Self {
        Self {     
            screen:                 [0u8; SCREEN_H * SCREEN_W],
            table_name:             [0u8; 2*1024], 
            table_palette:          [0u8; 32],
            table_pattern:          [0u8; 2*4096],    
            sprite_name_table:      [0u8; SCREEN_H*SCREEN_W*2],
            sprite_pattern_table:   [0u8; 128*128*2],
            scanline:        0, 
            cycle:           0,
            frame_complete: false,
            noise_state: 0x12345678,
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, colour: u8) {
        if x < SCREEN_W  && y < SCREEN_H {
            self.screen[y * SCREEN_W + x] = colour;
        }
    }

    
    
    pub fn clock(&mut self)  {
        // Test noise
        self.noise_state ^= self.noise_state << 13;
        self.noise_state ^= self.noise_state >> 17;
        self.noise_state ^= self.noise_state << 5;

        let c = (self.noise_state & 0xFF) as u8;

        self.set_pixel(self.cycle as usize, self.scanline as usize, c);
        
        // This is weird NES stuff
        // There are 341 PPU cycles per scanline
        self.cycle += 1;

        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline = self.scanline.wrapping_add(1);

            if self.scanline >= 261 {
                self.scanline = 0; // last value in u16 that wraps back to zero
                self.frame_complete = true;
            }
        }
    }

    pub fn get_frame_buffer(&self) -> Vec<u8> {
        self.screen.to_vec()
    }
}


impl PpuInterface for Olc2c02 {
    fn read_cpu(&mut self, addr: u16, _read_only: bool) -> u8 {
        let mut data : u8 = 0x00;
        let data = match addr {
         0x0000 => 0x00, // Control
         0x0001 => 0x00, // Mask
         0x0002 => 0x00, // Status
         0x0003 => 0x00, // OAM Address
         0x0004 => 0x00, // OAM Data
         0x0005 => 0x00, // Scroll
         0x0006 => 0x00, // PPU Address
         0x0007 => 0x00, // PPU Data
         _      => 0x00,
        };
        data
    }

    fn write_cpu(&mut self, addr: u16, data: u8)  {
        match addr {
         0x0000 => 0x00, // Control
         0x0001 => 0x00, // Mask
         0x0002 => 0x00, // Status
         0x0003 => 0x00, // OAM Address
         0x0004 => 0x00, // OAM Data
         0x0005 => 0x00, // Scroll
         0x0006 => 0x00, // PPU Address
         0x0007 => 0x00, // PPU Data
         _      => 0x00,
        };
    }

    fn read_ppu(&self, addr: u16, cartridge: &mut dyn CartridgeInterface) -> Option<u8> {
        let mut addr = addr & 0x3FFF;

        if let Some(data) = cartridge.read_ppu(addr) {
            return Some(data);
        } else if addr >= 0x0000 && addr <= 0x1FFF
        {
            // If the cartridge cant map the address, have
            // a physical location ready here
            let table = (addr & 0x1000) >> 12; // 0 or 1
            let offset = addr & 0x0FFF;        // 0..4095
            return Some(self.table_pattern[(table * 4096 + offset) as usize]);
        }
        else if addr >= 0x2000 && addr <= 0x3EFF
        {
            return None;
        }
        else if addr >= 0x3F00 && addr <= 0x3FFF
        {
            addr &= 0x001F;
            if addr == 0x0010 {addr = 0x0000};
            if addr == 0x0014 {addr = 0x0004};
            if addr == 0x0018 {addr = 0x0008};
            if addr == 0x001C {addr = 0x000C};
            return Some(self.table_palette[addr as usize]);
        }

        None

    }

    fn write_ppu(&mut self, addr: u16, data: u8, cartridge: &mut dyn CartridgeInterface) {
        let mut addr = addr & 0x3FFF;

        if let Some(_) = cartridge.write_ppu(addr & 0x3FFF, data) {

        } else if addr >= 0x0000 && addr <= 0x1FFF
        {
            let table = (addr & 0x1000) >> 12; // 0 or 1
            let offset = addr & 0x0FFF;        // 0..4095
            self.table_pattern[(table * 4096 + offset) as usize] = data;
        }
        else if addr >= 0x2000 && addr <= 0x3EFF
        {
        }
        else if addr >= 0x3F00 && addr <= 0x3FFF
        {
            addr &= 0x001F;
            if addr == 0x0010 {addr = 0x0000;}
            if addr == 0x0014 {addr = 0x0004;}
            if addr == 0x0018 {addr = 0x0008;}
            if addr == 0x001C {addr = 0x000C;}
            self.table_palette[addr as usize] = data;
        }
    }

}

impl Olc2c02 {

    pub fn get_pattern_table(&mut self, i: u8, palette: u8, cartridge: &mut dyn CartridgeInterface) -> &[u8; 128*128*2] {
        for n_tile_y in 0u16..16 {
            for n_tile_x in 0u16..16 {
                let n_offset = n_tile_y * 256u16 + n_tile_x * 16u16;

                for row in 0u16..8 {
                    let base = (i as u16) * 0x1000u16;

                    let Some(mut tile_lsb) =
                        self.read_ppu(base + n_offset + row + 0x0000u16, cartridge)
                    else {
                        continue;
                    };

                    let Some(mut tile_msb) =
                        self.read_ppu(base + n_offset + row + 0x0008u16, cartridge)
                    else {
                        continue;
                    };

                    for col in 0..8 {
                        // The tutorial code did not contain the bitshift, but during the review ChatGPT complained here
                        // I am not yet able to assess which one is correct
                        // I will run both and report back
                        //let pixel = (tile_lsb & 0x01) + (tile_msb & 0x01); 
                        let pixel = (tile_lsb & 0x01) | ((tile_msb & 0x01) << 1);
                        tile_lsb >>= 1;
                        tile_msb >>= 1; 

                        let x: u16 = n_tile_x * 8 + (7 - col);
                        let y: u16 = n_tile_y * 8 + row;
                        let w: u16 = 128;
                        let h: u16 = 128; 
                        let s: u16 = w * h;

                        // Map failed read to index 0
                        let index = match self.get_colour_from_palette_ram(palette, pixel, cartridge) {
                            Some(v) => v,
                            None => 0,
                        };

                        self.sprite_pattern_table[( (i as u16) * s + x + y * w) as usize] = index; 
                    }
                }

            }
        }
        
        &self.sprite_pattern_table
    }

    
	// This is a convenience function that takes a specified palette and pixel
	// index and returns the appropriate screen colour.
    fn get_colour_from_palette_ram(&self, palette: u8, pixel: u8, cartridge: &mut dyn CartridgeInterface) -> Option<u8> {
        let addr = 0x3F00u16 + ((palette as u16) << 2) + pixel as u16;
        self.read_ppu(addr, cartridge)
    }
}