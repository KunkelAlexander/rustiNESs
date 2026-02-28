use crate::interfaces::{CartridgeInterface, PpuInterface};

pub const SCREEN_W: usize = 256;
pub const SCREEN_H: usize = 240;

#[derive(Clone, Copy)]
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

pub struct Olc2c02 {
    frame:          [u8; SCREEN_H * SCREEN_W], // Frame buffer
    name_table:     [u8; 2*1024],              // 2 KB of physical VRAM for the name tables
    palette:        [u8; 32],                  // 32 Bytes physical VRAM for the palletes
    pattern:        [u8; 2*4096],              // 8 KB of physical VRAM for the patterns
    scanline:       u16, 
    cycle:          u16, 
    frame_complete: bool

}

impl Olc2c02 {
    pub fn new() -> Self {
        Self {     
            frame:           [0u8; SCREEN_H * SCREEN_W],
            name_table:      [0u8; 2*1024], 
            palette:         [0u8; 32],
            pattern:         [0u8; 2*4096],
            scanline:        0, 
            cycle:           0,
            frame_complete: false
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, colour: u8) {
        if x >= 0 && x < SCREEN_H  && y >= 0 && y <= SCREEN_W {
            self.frame[y * SCREEN_W + x] = colour;
        }
    }
    
    
    pub fn clock(&mut self)  {
        // Test noise
        let c = ((self.cycle + self.scanline) & 1) as u8;

        self.set_pixel((self.cycle - 1) as usize, self.scanline as usize, c);
        
        // This is weird NES stuff
        // There are 341 PPU cycles per scanline
        self.cycle += 1;

        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline = self.scanline.wrapping_add(1);

            if self.scanline >= 261 {
                self.scanline = 65535; // last value in u16 that wraps back to zero
                self.frame_complete = true;
            }
        }
    }

    pub fn is_frame_complete(&self) -> bool {
        self.frame_complete
    }

    pub fn get_frame_buffer(&self) -> Vec<u8> {
        self.frame.to_vec()
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
        cartridge.read_ppu(addr & 0x3FFF)
    }

    fn write_ppu(&mut self, addr: u16, data: u8, cartridge: &mut dyn CartridgeInterface) {
        cartridge.write_ppu(addr & 0x3FFF, data);
    }


}