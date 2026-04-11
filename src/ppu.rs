use crate::{cartridge, interfaces::{CartridgeInterface, PpuInterface}};

pub const SCREEN_W: usize = 256;
pub const SCREEN_H: usize = 240;

pub struct Olc2c02 {
    screen:                [u8; SCREEN_H*SCREEN_W],   // Frame buffer
    table_name:            [u8; 2*1024],              // 2 KB of physical VRAM for the name tables
    table_palette:         [u8; 32],                  // 32 Bytes physical VRAM for the palletes
    table_pattern:         [u8; 2*4096],              // 8 KB of physical VRAM for the patterns
    sprite_name_table:     [u8; SCREEN_H*SCREEN_W*2], // Helper for visualisation
    scanline:               u16, 
    cycle:                  u16, 
    pub frame_complete:     bool,
    noise_state:            u32,

    // registers
    status:                 u8,
    mask:                   u8,
    control:                u8,
    loopy:                  u16,

    address_latch:          u8, 
    ppu_data_buffer:        u8, 
    ppu_address:            u16,
    pub nmi:                bool
}

impl Olc2c02 {
    // =====================
    // STATUS (0x2002)
    // =====================
    pub const STATUS_UNUSED:                u8 = 0b0001_1111;
    pub const STATUS_SPRITE_OVERFLOW:       u8 = 1 << 5;
    pub const STATUS_SPRITE_ZERO_HIT:       u8 = 1 << 6;
    pub const STATUS_VERTICAL_BLANK:        u8 = 1 << 7;

    // =====================
    // MASK (0x2001)
    // =====================
    pub const MASK_GRAYSCALE:               u8 = 1 << 0;
    pub const MASK_RENDER_BACKGROUND_LEFT:  u8 = 1 << 1;
    pub const MASK_RENDER_SPRITES_LEFT:     u8 = 1 << 2;
    pub const MASK_RENDER_BACKGROUND:       u8 = 1 << 3;
    pub const MASK_RENDER_SPRITES:          u8 = 1 << 4;
    pub const MASK_ENHANCE_RED:             u8 = 1 << 5;
    pub const MASK_ENHANCE_GREEN:           u8 = 1 << 6;
    pub const MASK_ENHANCE_BLUE:            u8 = 1 << 7;

    // =====================
    // CONTROL (0x2000)
    // =====================
    pub const CTRL_NAMETABLE_X:             u8 = 1 << 0;
    pub const CTRL_NAMETABLE_Y:             u8 = 1 << 1;
    pub const CTRL_INCREMENT_MODE:          u8 = 1 << 2;
    pub const CTRL_PATTERN_SPRITE:          u8 = 1 << 3;
    pub const CTRL_PATTERN_BACKGROUND:      u8 = 1 << 4;
    pub const CTRL_SPRITE_SIZE:             u8 = 1 << 5;
    pub const CTRL_SLAVE_MODE:              u8 = 1 << 6;
    pub const CTRL_ENABLE_NMI:              u8 = 1 << 7;

    // =====================
    // LOOPY REGISTER (u16)
    // =====================
    pub const LOOPY_COARSE_X_MASK:          u16 = 0b00000_00000_00000_11111;
    pub const LOOPY_COARSE_Y_MASK:          u16 = 0b00000_00000_11111_00000;
    pub const LOOPY_NAMETABLE_X:            u16 = 1 << 10;
    pub const LOOPY_NAMETABLE_Y:            u16 = 1 << 11;
    pub const LOOPY_FINE_Y_MASK:            u16 = 0b111 << 12;

    pub fn new() -> Self {
        Self {     
            screen:                 [0u8; SCREEN_H * SCREEN_W],
            table_name:             [0u8; 2*1024], 
            table_palette:          [0u8; 32],
            table_pattern:          [0u8; 2*4096],    
            sprite_name_table:      [0u8; SCREEN_H*SCREEN_W*2],
            scanline:        0, 
            cycle:           0,
            frame_complete:  false,
            noise_state:     0x12345678,
            status:          0x00,
            mask:            0x00,
            control:         0x00,
            loopy:           0x0000,
            address_latch:   0x00, 
            ppu_data_buffer: 0x00, 
            ppu_address:     0x0000,
            nmi:             false
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, colour: u8) {
        if x < SCREEN_W  && y < SCREEN_H {
            self.screen[y * SCREEN_W + x] = colour;
        }
    }

    
    // This advances the PPU
    // Visible scanlines: 0 ... 239 
    // Post-render: 240
    // Vblank: 241...260
    // Pre-render: 261 
    // Javidx9 uses -1 for pre-render since he uses a signed integer
    pub fn clock(&mut self)  {

        if self.scanline == 261 && self.cycle == 1 {
            self.status &= !Olc2c02::STATUS_VERTICAL_BLANK;
        }

        if self.scanline == 241 && self.cycle == 1 {
            self.status |= Olc2c02::STATUS_VERTICAL_BLANK;
            if self.control & Olc2c02::CTRL_ENABLE_NMI != 0 {
                self.nmi = true;
            }
        }
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

            if self.scanline >= 262 {
                self.scanline = 0;
                self.frame_complete = true;
            }
        }
    }

    pub fn get_frame_buffer(&self) -> Vec<u8> {
        self.screen.to_vec()
    }

    // Depending on the increment mode flag, we either move horizontally (1 tile) or vertically (skip 32 tiles horizontally)
    fn ppu_addr_increment(&self) -> u16 {
        if (self.control & Olc2c02::CTRL_INCREMENT_MODE) != 0 {
            32
        } else {
            1
        }
    }
}


impl PpuInterface for Olc2c02 {
    fn read_cpu(&mut self, addr: u16, _read_only: bool, cartridge: &mut dyn CartridgeInterface) -> u8 {
    
        let data = match addr {
         0x0000 => 0x00, // Control
         0x0001 => 0x00, // Mask
         0x0002 => {
            let temp = (self.status & 0xE0) | (self.ppu_data_buffer & 0x1F);
            self.status &= !Olc2c02::STATUS_VERTICAL_BLANK;
            self.address_latch = 0; 
            temp
         }, // Status
         0x0003 => 0x00, // OAM Address
         0x0004 => 0x00, // OAM Data
         0x0005 => 0x00, // Scroll
         0x0006 => 0x00, // PPU Address
         0x0007 => {
            let temp         = self.ppu_data_buffer;
            let addr        = self.ppu_address;
            self.ppu_data_buffer = self.read_ppu(addr, cartridge).unwrap_or(0x00);;

            // Auto-increment for convenience - we rarely want to read/write the same location twice
            self.ppu_address = self.ppu_address.wrapping_add(self.ppu_addr_increment()) & 0x3FFF;

            if addr >= 0x3F00 {
                self.ppu_data_buffer
            } else {
                temp
            }
         }, // PPU Data
         _      => 0x00,
        };
        data
    }

    fn write_cpu(&mut self, addr: u16, data: u8, cartridge: &mut dyn CartridgeInterface)  {
        match addr {
        // Control
         0x0000 => {
            self.control = data;
         }, 
         // Mask
         0x0001 => {
            self.mask = data;
         }, 
         0x0002 => {}, // Status
         0x0003 => {}, // OAM Address
         0x0004 => {}, // OAM Data
         0x0005 => {}, // Scroll
         // PPU Address
         0x0006 => {
            if self.address_latch == 0 {
                self.ppu_address = (self.ppu_address & 0x00FF) | (((data & 0x3F) as u16) << 8); 
                self.address_latch = 1;
            } else {
                self.ppu_address = (self.ppu_address & 0xFF00) | data as u16; 
                self.address_latch = 0;
            }
         }, 
         // PPU Data
         0x0007 => {
            self.write_ppu(self.ppu_address, data, cartridge);
            
            // Auto-increment for convenience - we rarely want to read/write the same location twice
            self.ppu_address = self.ppu_address.wrapping_add(self.ppu_addr_increment()) & 0x3FFF;
         }, 
         _      => {},
        };
    }

    fn read_ppu(&self, addr: u16, cartridge: &dyn CartridgeInterface) -> Option<u8> {
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
            // cartridge handles mirroring
            return Some(self.table_name[cartridge.map_nametable_addr(addr) as usize])
            
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
            // cartridge handles mirroring
            self.table_name[cartridge.map_nametable_addr(addr) as usize] = data;
        }
        else if addr >= 0x3F00 && addr <= 0x3FFF
        {
            let raw = addr;
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

    pub fn get_name_table(&self) -> Vec<u8> {
        self.table_name[..1024].to_vec()
    }

    pub fn get_pattern_table(&self, i: u8, palette: u8, cartridge: &dyn CartridgeInterface) -> Vec<u8> {
        
        let mut sprite_pattern_table = [0u8; 128*128];

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
                        // let pixel = (tile_lsb & 0x01) + (tile_msb & 0x01); 
                        // let pixel = (tile_lsb & 0x01) | ((tile_msb & 0x01) << 1);
                        // Turns that we need the addition to get the right output

                        let pixel = (tile_lsb & 0x01) + (tile_msb & 0x01); 
                        tile_lsb >>= 1;
                        tile_msb >>= 1; 

                        let x: u16 = n_tile_x * 8 + (7 - col);
                        let y: u16 = n_tile_y * 8 + row;
                        let w: u16 = 128;
                        

                        // Map failed read to index 0
                        let index = match self.get_colour_from_palette_ram(palette, pixel, cartridge) {
                            Some(v) => {
                                v
                            },
                            None => {
                                0
                            },
                        };

                        sprite_pattern_table[(x + y * w) as usize] = index; 
                    }
                }

            }
        }
        
        sprite_pattern_table.to_vec()
    }

    
	// This is a convenience function that takes a specified palette and pixel
	// index and returns the appropriate screen colour.
    fn get_colour_from_palette_ram(&self, palette: u8, pixel: u8, cartridge: &dyn CartridgeInterface) -> Option<u8> {
        let addr = 0x3F00u16 + ((palette as u16) << 2) + pixel as u16;
        self.read_ppu(addr, cartridge)
    }
}