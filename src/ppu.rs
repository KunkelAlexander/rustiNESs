use crate::{interfaces::{CartridgeInterface, PpuInterface}};

use std::marker::PhantomData;
use serde::{Serialize, Deserialize};

pub const SCREEN_W: usize = 256;
pub const SCREEN_H: usize = 240;


// Javidx9 goes via bitfields here but the bit gymnastics in Rust are bit too much for me
// The following is much nicer than operating on a single u8 in Rust 
#[derive(Copy, Clone, Default, Serialize, Deserialize)]
struct Loopy {
    addr: u16,
}

impl Loopy {
    fn coarse_x        (&self) -> u8      { ((self.addr >>  0) & 0x1F) as u8 }
    fn coarse_y        (&self) -> u8      { ((self.addr >>  5) & 0x1F) as u8 }
    fn nametable_x     (&self) -> u8      { ((self.addr >> 10) & 0x01) as u8 }
    fn nametable_y     (&self) -> u8      { ((self.addr >> 11) & 0x01) as u8 }
    fn fine_y          (&self) -> u8      { ((self.addr >> 12) & 0x07) as u8 }

    fn set_coarse_x    (&mut self, v: u8) { self.addr = (self.addr & !(0x1F <<  0)) | ((v as u16) <<  0); }
    fn set_coarse_y    (&mut self, v: u8) { self.addr = (self.addr & !(0x1F <<  5)) | ((v as u16) <<  5); }
    fn set_nametable_x (&mut self, v: u8) { self.addr = (self.addr & !(0x01 << 10)) | ((v as u16) << 10); }
    fn set_nametable_y (&mut self, v: u8) { self.addr = (self.addr & !(0x01 << 11)) | ((v as u16) << 11); }
    fn set_fine_y      (&mut self, v: u8) { self.addr = (self.addr & !(0x07 << 12)) | ((v as u16) << 12); }

    fn to_u16(&self) -> u16 { self.addr }                    // was 5 loads + 5 shifts + 4 ORs
    fn from_u16(v: u16) -> Self { Loopy {addr: v} }          // was 5 stores
}

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
struct Sprite {
    x:         u8, 
    y:         u8,
    id:        u8, // tile id from pattern memory
    attribute: u8, // how should sprite be rendered
}

impl Sprite {
    fn read_byte(&self, offset: u8) -> u8 {
        match offset {
            0 => self.y,
            1 => self.id,
            2 => self.attribute,
            3 => self.x,
            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, offset: u8, data: u8) {
        match offset {
            0 => self.y         = data,
            1 => self.id        = data,
            2 => self.attribute = data,
            3 => self.x         = data,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SpriteArray<const N: usize> {
    sprites: Vec<Sprite>,
}

impl<const N: usize> Default for SpriteArray<N> {
    fn default() -> Self {
        Self {
            sprites: vec![Sprite::default(); N],
        }
    }
}

impl<const N: usize> SpriteArray<N> {

    pub fn write(&mut self, addr: u8, data: u8) {
        let index = (addr / 4) as usize;
        let offset   =  addr % 4;

        self.sprites[index].write_byte(offset, data);
    }

    pub fn read(&self, addr: u8) -> u8 {
        let index = (addr / 4) as usize;
        let offset   =  addr % 4;

        self.sprites[index].read_byte(offset)
    }
}

pub type OAM = SpriteArray<64>;
pub type SpriteScanline = SpriteArray<8>; 

fn default_screen() -> Vec<u8> {
    vec![0u8; SCREEN_H * SCREEN_W]
}

#[derive(Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Olc2c02<C: CartridgeInterface> {
    #[serde(skip, default = "default_screen")]
    screen:                Vec<u8>,  // Frame buffer
    table_name:            Vec<u8>,  // 2 KB of physical VRAM for the name tables
    table_palette:         [u8; 32], // 32 Bytes physical VRAM for the palletes
    table_pattern:         Vec<u8>,  // 8 KB of physical VRAM for the patterns
    scanline:               u16, 
    cycle:                  u16, 
    pub frame_complete:     bool,
    noise_state:            u32,

    // Registers
    status:                 u8,
    mask:                   u8,
    control:                u8,
    vram_addr:              Loopy,
    tram_addr:              Loopy,
    fine_x:                 u8,

    address_latch:          u8, 
    ppu_data_buffer:        u8, 
    pub nmi:                bool, 

	// Background rendering
    bg_shifter_pattern_hi:  u16,
    bg_shifter_pattern_lo:  u16,
    bg_shifter_attrib_hi:   u16,
    bg_shifter_attrib_lo:   u16,
    bg_next_tile_lsb:       u8,
    bg_next_tile_msb:       u8,
    bg_next_tile_id:        u8,
    bg_next_tile_attrib:    u8,

    // Sprite memory is public because we need access from the bus for the DMA operation
    pub oam:               OAM,
    pub oam_addr:          u8,

    sprite_scanline:       SpriteScanline,
    sprite_count:          u8,
    sp_shifter_pattern_lo: [u8; 8], 
    sp_shifter_pattern_hi: [u8; 8],

    // There is a single flag that indicates whether a sprite overlaps with a background tile
    // This is only done for one sprite - sprite 0
    // This is done for synchronising the PPU with the CPU 
    // The CPU periodically examines the sprite zero hit flag periodically and gets an idea of how far down the scanline has gone
    // This is important for status bars - a static bar is rendered up to a given scanline and then the game is rendered normally below
    // The sprite 0 tells us where to start normal rendering
    b_sp_0_being_rendered: bool,
    b_sp_0_hit_possible:   bool,

    
    #[serde(skip)]
    _cartridge: PhantomData<C>,
}


// masks for self.status
//const STATUS_UNUSED:                u8 = 0b0001_1111;
const STATUS_SPRITE_OVERFLOW:       u8 = 1 << 5;
const STATUS_SPRITE_ZERO_HIT:       u8 = 1 << 6;
const STATUS_VERTICAL_BLANK:        u8 = 1 << 7;

// masks for self.mask
//const MASK_GRAYSCALE:               u8 = 1 << 0;
const MASK_RENDER_BACKGROUND_LEFT:  u8 = 1 << 1;
const MASK_RENDER_SPRITES_LEFT:     u8 = 1 << 2;
const MASK_RENDER_BACKGROUND:       u8 = 1 << 3;
const MASK_RENDER_SPRITES:          u8 = 1 << 4;
//const MASK_ENHANCE_RED:             u8 = 1 << 5;
//const MASK_ENHANCE_GREEN:           u8 = 1 << 6;
//const MASK_ENHANCE_BLUE:            u8 = 1 << 7;

// masks for self.control
const CTRL_NAMETABLE_X:             u8 = 1 << 0;
const CTRL_NAMETABLE_Y:             u8 = 1 << 1;
const CTRL_INCREMENT_MODE:          u8 = 1 << 2;
const CTRL_PATTERN_SPRITE:          u8 = 1 << 3;
const CTRL_PATTERN_BACKGROUND:      u8 = 1 << 4;
const CTRL_SPRITE_SIZE:             u8 = 1 << 5;
//const CTRL_SLAVE_MODE:              u8 = 1 << 6;
const CTRL_ENABLE_NMI:              u8 = 1 << 7;


impl<C: CartridgeInterface> Olc2c02<C> {


    pub fn new() -> Self {
        Self {     
            screen:                 vec![0u8; SCREEN_H * SCREEN_W],
            table_name:             vec![0u8; 2*1024],
            table_palette:          [0x00; 32],
            table_pattern:          vec![0u8; 2*4096],
            scanline:                0, 
            cycle:                   0,
            frame_complete:          false,
            noise_state:             0x12345678,
            status:                  0x00,
            mask:                    0x00,
            control:                 0x00,
            vram_addr:               Loopy::default(),
            tram_addr:               Loopy::default(),
            fine_x:                  0x00,
            address_latch:           0x00, 
            ppu_data_buffer:         0x00, 
            nmi:                     false, 
            bg_shifter_pattern_hi:   0x0000,
            bg_shifter_pattern_lo:   0x0000,
            bg_shifter_attrib_hi:    0x0000,
            bg_shifter_attrib_lo:    0x0000,
            bg_next_tile_lsb:        0x00,
            bg_next_tile_msb:        0x00,
            bg_next_tile_id:         0x00,
            bg_next_tile_attrib:     0x00,
            oam:                     OAM::default(),
            oam_addr:                0x00,
            sprite_scanline:         SpriteScanline::default(),
            sprite_count:            0x00,
            sp_shifter_pattern_hi:  [0x0000; 8],
            sp_shifter_pattern_lo:  [0x0000; 8],
            b_sp_0_being_rendered:   false,
            b_sp_0_hit_possible:     false,

            
            _cartridge:             PhantomData,
        }
    }

    
    
    // This advances the PPU
    // Visible scanlines: 0 ... 239 
    // Post-render: 240
    // Vblank: 241...260
    // Pre-render: 261 
    // Javidx9 uses -1 for pre-render since he uses a signed integer
    pub fn clock(&mut self, cartridge: &mut C)  {

        let render_scanline = self.scanline < 240 || self.scanline == 261;

        if render_scanline && ((self.cycle >= 2 && self.cycle < 258) || (self.cycle >= 321 && self.cycle < 338)) {
            self.fetch_background_tile(cartridge);
        }

        if render_scanline && self.cycle == 256 {
            self.increment_scroll_y();
        }

        if render_scanline && self.cycle == 257 {
            self.load_background_shifters();
            self.transfer_address_x();
        }

        if render_scanline && (self.cycle == 338 || self.cycle == 340) {
            let addr = 0x2000 | (self.vram_addr.to_u16() & 0x0FFF);
            self.bg_next_tile_id = self.read_ppu(addr, cartridge).unwrap_or(0);
        }

        if render_scanline && (self.cycle == 257 && self.scanline < 240) {
            self.evaluate_sprites();
        }

        if render_scanline && self.cycle == 340 {
            self.load_sprite_patterns(cartridge);
        }

        if self.scanline == 241 && self.cycle == 1 {
            self.status |= STATUS_VERTICAL_BLANK;
            if self.control & CTRL_ENABLE_NMI != 0 {
                self.nmi = true;
            }
        }

        if self.scanline == 261 && self.cycle >= 280 && self.cycle < 305 {
            self.transfer_address_y();
        }

        // Effectively start of new frame
        if self.scanline == 261 && self.cycle == 1 {
            self.status &= !STATUS_VERTICAL_BLANK;
            self.status &= !STATUS_SPRITE_OVERFLOW;
            self.status &= !STATUS_SPRITE_ZERO_HIT;

            for i in 0u8..8 {
                self.sp_shifter_pattern_hi[i as usize] = 0;
                self.sp_shifter_pattern_lo[i as usize] = 0;
            }
        }

        if self.scanline < 240 && self.cycle >= 1 && self.cycle <= 256 {
            self.compose_pixel(cartridge);
        }

        // This is weird NES stuff
        // There are 341 PPU cycles per scanline
        self.cycle += 1;

        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;

            if self.scanline >= 262 {
                self.scanline = 0;
                self.frame_complete = true;
            }
        }
    }

    
    
    pub fn set_pixel(&mut self, x: usize, y: usize, colour: u8) {
        self.screen[y * SCREEN_W + x] = colour;
    }

	// Increment the background tile "pointer" one tile/column horizontally
    fn increment_scroll_x(&mut self) {
        if (self.mask & MASK_RENDER_BACKGROUND != 0) || (self.mask & MASK_RENDER_SPRITES != 0) {
            if self.vram_addr.coarse_x() == 31 {
                self.vram_addr.set_coarse_x(0);
                self.vram_addr.set_nametable_x(self.vram_addr.nametable_x() ^ 1);
            } else {
                self.vram_addr.set_coarse_x(self.vram_addr.coarse_x() + 1);
            }
        }
    }

	// Increment the background tile "pointer" one scanline vertically
    fn increment_scroll_y(&mut self) {
        if (self.mask & MASK_RENDER_BACKGROUND != 0) || (self.mask & MASK_RENDER_SPRITES != 0) {
            if self.vram_addr.fine_y() < 7 {
                self.vram_addr.set_fine_y(self.vram_addr.fine_y() + 1);
            } else {
                self.vram_addr.set_fine_y(0);

                if self.vram_addr.coarse_y() == 29 {
                    self.vram_addr.set_coarse_y(0);
                    self.vram_addr.set_nametable_y(self.vram_addr.nametable_y() ^ 1);
                } else if self.vram_addr.coarse_y() == 31 {
                    self.vram_addr.set_coarse_y(0);
                } else {
                    self.vram_addr.set_coarse_y(self.vram_addr.coarse_y() + 1);
                }
            }
        }
    }

    // Transfer temporarily stored horizontal nametable access information into the main pointer
    fn transfer_address_x(&mut self) {
        if (self.mask & MASK_RENDER_BACKGROUND != 0) || (self.mask & MASK_RENDER_SPRITES != 0) {
            self.vram_addr.set_nametable_x(self.tram_addr.nametable_x());
            self.vram_addr.set_coarse_x   (self.tram_addr.coarse_x());
        }
    }

    // Transfer temporarily stored vertical nametable access information into the main pointer
    fn transfer_address_y(&mut self) {
        if (self.mask & MASK_RENDER_BACKGROUND != 0) || (self.mask & MASK_RENDER_SPRITES != 0) {
            self.vram_addr.set_nametable_y(self.tram_addr.nametable_y());
            self.vram_addr.set_coarse_y   (self.tram_addr.coarse_y());
            self.vram_addr.set_fine_y     (self.tram_addr.fine_y());
        }
    }

    // Prepare the background tile shifters for outputting next 8 pixels in scanline
    fn load_background_shifters(&mut self) {
		self.bg_shifter_pattern_lo = (self.bg_shifter_pattern_lo & 0xFF00) | self.bg_next_tile_lsb as u16;
		self.bg_shifter_pattern_hi = (self.bg_shifter_pattern_hi & 0xFF00) | self.bg_next_tile_msb as u16;
		self.bg_shifter_attrib_lo  = (self.bg_shifter_attrib_lo  & 0xFF00) | if (self.bg_next_tile_attrib & 0b01) != 0 { 0x00FF } else { 0x0000 };
		self.bg_shifter_attrib_hi  = (self.bg_shifter_attrib_hi  & 0xFF00) | if (self.bg_next_tile_attrib & 0b10) != 0 { 0x00FF } else { 0x0000 };
    }

    
    // Every cycle the shifters shift their contents by 1 bit because the output progresses by 1 pixel
    fn update_shifters(&mut self) {
        if self.mask & MASK_RENDER_BACKGROUND != 0 {
            self.bg_shifter_pattern_lo <<= 1;
            self.bg_shifter_pattern_hi <<= 1;
            self.bg_shifter_attrib_lo  <<= 1;
            self.bg_shifter_attrib_hi  <<= 1;
        }

        // We want to detect when the scanline collides with the sprite
        // To do so, we decrement the sprite's x position every cycle 
        // One x = 0, we know that the scanline has reached it and that it should be rendered
        if (self.mask & MASK_RENDER_SPRITES != 0) && self.cycle >= 1 && self.cycle < 258 {
            for i in 0u8..self.sprite_count {
                let sprite = &mut self.sprite_scanline.sprites[i as usize]; 
                if sprite.x > 0 {
                    sprite.x -= 1; 
                } else {
                    self.sp_shifter_pattern_lo[i as usize] <<= 1;
                    self.sp_shifter_pattern_hi[i as usize] <<= 1;
                }
            }
        }
    }

    
    fn fetch_background_tile(&mut self, cartridge: &mut C) {
        self.update_shifters();

        match (self.cycle - 1) % 8 {
            0 => {
                self.load_background_shifters();

                let addr = 0x2000 | (self.vram_addr.to_u16() & 0x0FFF);

                self.bg_next_tile_id = self.read_ppu(addr, cartridge).unwrap_or(0);
            },
            2 => {
                let addr = (0x23C0 |  ((self.vram_addr.nametable_y() as u16) << 11)
                                        |  ((self.vram_addr.nametable_x() as u16) << 10)
                                        | (((self.vram_addr.coarse_y()    as u16) >> 2) << 3))
                                        |  ((self.vram_addr.coarse_x()    as u16) >> 2);
                self.bg_next_tile_attrib = self.read_ppu(addr, cartridge).unwrap_or(0);
                if self.vram_addr.coarse_y() & 0x02 != 0 {self.bg_next_tile_attrib >>= 4;}
                if self.vram_addr.coarse_x() & 0x02 != 0 {self.bg_next_tile_attrib >>= 2;}
                self.bg_next_tile_attrib &= 0x03;
            }
            4 => {
                let addr = (((((self.control & CTRL_PATTERN_BACKGROUND) as u16) >> 4) << 12) as u16)
                              + ((self.bg_next_tile_id as u16) << 4)
                              + (self.vram_addr.fine_y() as u16);

                self.bg_next_tile_lsb = self.read_ppu(addr, cartridge).unwrap_or(0);
            }

            6 => {
                let addr = (((((self.control & CTRL_PATTERN_BACKGROUND) as u16) >> 4) << 12) as u16)
                              + ((self.bg_next_tile_id as u16) << 4)
                              + (self.vram_addr.fine_y() as u16)
                              + 8;

                self.bg_next_tile_msb = self.read_ppu(addr, cartridge).unwrap_or(0);
            }
            7 => {
                self.increment_scroll_x();
            }
            _ => {}
        }
    }

    fn evaluate_sprites(&mut self) {
        self.sprite_count = 0;
        self.sprite_scanline.sprites.fill(Sprite::default());
        self.sp_shifter_pattern_lo.fill(0);
        self.sp_shifter_pattern_hi.fill(0);

        self.status &= !STATUS_SPRITE_OVERFLOW;

        let sprite_size: i16 = if (self.control & CTRL_SPRITE_SIZE) != 0 {16} else {8};

        self.b_sp_0_hit_possible = false;

        for n_oam_entry in 0u8..64 {
            let oam_sprite = self.oam.sprites[n_oam_entry as usize];
            let diff = self.scanline as i16 - oam_sprite.y as i16;

            if diff >= 0 && diff < sprite_size {
                // If we don't already have 8 sprites, we copy the sprite information from the OAM into the sprite scanline array
                if self.sprite_count < 8 {
                    // Is this sprite sprite zero?
                    if n_oam_entry == 0 {
                        self.b_sp_0_hit_possible = true;
                    }
                    self.sprite_scanline.sprites[self.sprite_count as usize] = self.oam.sprites[n_oam_entry as usize];
                    self.sprite_count += 1;
                } else {
                    self.status |=  STATUS_SPRITE_OVERFLOW;
                    break;

                }
            }
        }
    }

    fn load_sprite_patterns(&mut self, cartridge: &mut C) {
        for i in 0u8..self.sprite_count {

            let mut sprite_pattern_bits_lo: u8;
            let mut sprite_pattern_bits_hi: u8;
            let sprite_pattern_addr_lo: u16;
            let sprite_pattern_addr_hi: u16;

            let sprite = self.sprite_scanline.sprites[i as usize];


            // 8x8 sprite mode
            if (self.control & CTRL_SPRITE_SIZE) == 0 {
                // These 3 indices index into a 2D memory structure
                // This is either 0k or 4k offset on the CPU bus
                let offset1 = (((self.control & CTRL_PATTERN_SPRITE) != 0) as u16) << 12;
                // << 4 = * 16 = each tile is 16 bytes in size
                let offset2 = (sprite.id as u16) << 4;
                // Which row of the tile are we currently in - Unsigned here because it should always be positive
                let offset3 = self.scanline - (sprite.y as u16);

                // Sprite is not flipped vertically, i.e. normal
                if sprite.attribute & 0x80 == 0 {
                    sprite_pattern_addr_lo = offset1 | offset2 | offset3;

                }
                // Sprite is flipped vertically
                // Sprite is not flipped vertically, i.e. normal
                else
                {
                    sprite_pattern_addr_lo = offset1 | offset2 | (7 - offset3);
                }
            }
            // 8x16 sprite mode
            else
            {

                // Sprite is not flipped vertically, i.e. normal
                if sprite.attribute & 0x80 == 0 {

                    // Read top half tile
                    if self.scanline - (sprite.y as u16) < 8 {

                        let offset1 =  ((sprite.id & 0x01) as u16)      << 12;
                        let offset2 = (((sprite.id & 0xFE) as u16) + 0) <<  4;
                        let offset3 = (self.scanline - (sprite.y as u16)) & 0x0007;

                        sprite_pattern_addr_lo = offset1 | offset2 | offset3;
                    }
                    // Read bottom half tile
                    else
                    {
                        let offset1 =  ((sprite.id & 0x01) as u16)      << 12;
                        let offset2 = (((sprite.id & 0xFE) as u16) + 1) <<  4;
                        let offset3 = (self.scanline - (sprite.y as u16)) & 0x0007;

                        sprite_pattern_addr_lo = offset1 | offset2 | offset3;

                    }

                }
                // Sprite is flipped vertically
                // Sprite is not flipped vertically, i.e. normal
                else
                {

                    // Read top half tile
                    if self.scanline - (sprite.y as u16) < 8 {

                        let offset1 =  ((sprite.id & 0x01) as u16)      << 12;
                        let offset2 = (((sprite.id & 0xFE) as u16) + 1) <<  4;
                        let offset3 = 7 - ((self.scanline - (sprite.y as u16)) & 0x0007);

                        sprite_pattern_addr_lo = offset1 | offset2 | offset3;
                    }
                    // Read bottom half tile
                    else
                    {
                        let offset1 =  ((sprite.id & 0x01) as u16)      << 12;
                        let offset2 = (((sprite.id & 0xFE) as u16) + 0) <<  4;
                        let offset3 = 7  - ((self.scanline - (sprite.y as u16)) & 0x0007);

                        sprite_pattern_addr_lo = offset1 | offset2 | offset3;

                    }
                }

            } // End of if for setting sprite_pattern_addr_lo

            sprite_pattern_addr_hi = sprite_pattern_addr_lo + 8;

            sprite_pattern_bits_lo = self.read_ppu(sprite_pattern_addr_lo, cartridge).unwrap_or(0);
            sprite_pattern_bits_hi = self.read_ppu(sprite_pattern_addr_hi, cartridge).unwrap_or(0);

            // if the sprite is flipped horizontally, we need to flip the pattern bytes
            if sprite.attribute & 0x40 != 0 {
                sprite_pattern_bits_lo = sprite_pattern_bits_lo.reverse_bits();
                sprite_pattern_bits_hi = sprite_pattern_bits_hi.reverse_bits();
            }


			self.sp_shifter_pattern_lo[i as usize] = sprite_pattern_bits_lo;
			self.sp_shifter_pattern_hi[i as usize] = sprite_pattern_bits_hi;
        } // End of the loop over the active sprites
    }

    fn compose_pixel(&mut self, cartridge: &mut C) {

        // Background
        let mut bg_pixel:   u8 = 0x00;
        let mut bg_palette: u8 = 0x00;


        if self.mask & MASK_RENDER_BACKGROUND != 0 {
            let bit_mux: u16 = 0x8000 >> self.fine_x;


            // Select Plane pixels by extracting from the shifter
            // at the required location.
            let p0_pixel = ((self.bg_shifter_pattern_lo & bit_mux) > 0) as u8;
            let p1_pixel = ((self.bg_shifter_pattern_hi & bit_mux) > 0) as u8;

            // Combine to form pixel index
            bg_pixel         = (p1_pixel << 1) | p0_pixel;

            // Get palette
            let bg_pal0  = ((self.bg_shifter_attrib_lo & bit_mux) > 0) as u8;
            let bg_pal1  = ((self.bg_shifter_attrib_hi & bit_mux) > 0) as u8;
            bg_palette       = (bg_pal1 << 1) | bg_pal0;

        }


        // Foreground
        let mut fg_pixel:    u8 = 0x00;
        let mut fg_palette:  u8 = 0x00;
        let mut fg_priority: bool = false;

        if self.mask & MASK_RENDER_SPRITES != 0  {
            self.b_sp_0_being_rendered = false;

            for i in 0u8..self.sprite_count {
                let sprite = self.sprite_scanline.sprites[i as usize];

                // Scanline has collided with the sprite
                if sprite.x == 0 {

                    let fg_pixel_lo: u8 = ((self.sp_shifter_pattern_lo[i as usize] & 0x80) > 0) as u8;
                    let fg_pixel_hi: u8 = ((self.sp_shifter_pattern_hi[i as usize] & 0x80) > 0) as u8;
                    fg_pixel = (fg_pixel_hi << 1) | fg_pixel_lo;


                    fg_palette  = (sprite.attribute & 0x03) + 0x04;
                    fg_priority = (sprite.attribute & 0x20) == 0;

                    if fg_pixel != 0 {
                        if i == 0 {
                            self.b_sp_0_being_rendered = true;
                        }

                        break;
                    }
                }
            } // Loop over sprites that we break out of
        } // Check whether sprites should be rendered


        // Combine background and foreground pixel
        let mut pixel:   u8 = 0x00;
        let mut palette: u8 = 0x00;

        if bg_pixel == 0 && fg_pixel == 0 {
            pixel   = 0x00;
            palette = 0x00;
        } else if bg_pixel == 0 && fg_pixel > 0 {
            pixel   = fg_pixel;
            palette = fg_palette
        } else if bg_pixel > 0 && fg_pixel == 0 {
            pixel   = bg_pixel;
            palette = bg_palette;
        } else if bg_pixel > 0 && fg_pixel > 0 {
            if fg_priority {
                pixel   = fg_pixel;
                palette = fg_palette;
            } else {
                pixel   = bg_pixel;
                palette = bg_palette;
            }

            if self.b_sp_0_being_rendered && self.b_sp_0_hit_possible {
                if ((self.mask & MASK_RENDER_BACKGROUND) != 0) && ((self.mask & MASK_RENDER_SPRITES) != 0) {
                    let left_edge_enabled =
                        (self.mask & MASK_RENDER_BACKGROUND_LEFT) != 0 &&
                        (self.mask & MASK_RENDER_SPRITES_LEFT)    != 0;

                    if !left_edge_enabled {
                        if self.cycle >= 9 && self.cycle < 258 {
                            self.status |= STATUS_SPRITE_ZERO_HIT;
                        }
                    } else {
                        if self.cycle >= 1 && self.cycle < 258 {
                            self.status |= STATUS_SPRITE_ZERO_HIT;
                        }
                    }
                }
            }
        }


        let colour = self.get_colour_from_palette_ram(palette, pixel, cartridge).unwrap_or(0);

        self.set_pixel((self.cycle - 1) as usize, self.scanline as usize, colour);
    }


    pub fn get_frame_buffer(&self) -> *const u8 {
        self.screen.as_ptr()
    }

    pub fn get_frame_buffer_len(&self) -> usize { 
        self.screen.len()
    }


    // Depending on the increment mode flag, we either move horizontally (1 tile) or vertically (skip 32 tiles horizontally)
    fn ppu_addr_increment(&self) -> u16 {
        if (self.control & CTRL_INCREMENT_MODE) != 0 {
            32
        } else {
            1
        }
    }

    pub fn reset(&mut self) {
        self.scanline               = 0;
        self.cycle                  = 0;
        self.frame_complete         = false;
        self.noise_state            = 0x12345678;
        self.status                 = 0x00;
        self.mask                   = 0x00;
        self.control                = 0x00;
        self.vram_addr              = Loopy::default();
        self.tram_addr              = Loopy::default();
        self.fine_x                 = 0x00;
        self.address_latch          = 0x00; 
        self.ppu_data_buffer        = 0x00; 
        self.nmi                    = false; 
        self.bg_shifter_pattern_hi  = 0x0000;
        self.bg_shifter_pattern_lo  = 0x0000;
        self.bg_shifter_attrib_hi   = 0x0000;
        self.bg_shifter_attrib_lo   = 0x0000;
        self.bg_next_tile_lsb       = 0x00;
        self.bg_next_tile_msb       = 0x00;
        self.bg_next_tile_id        = 0x00;
        self.bg_next_tile_attrib    = 0x00;
        self.oam                    = OAM::default();
        self.oam_addr               = 0x00;
        self.sprite_scanline.sprites.fill(Sprite::default());
        self.sprite_count           = 0x00;
        self.sp_shifter_pattern_hi  = [0x00; 8];
        self.sp_shifter_pattern_lo  = [0x00; 8];
    }
}


impl<C: CartridgeInterface> PpuInterface<C> for Olc2c02<C> {
    fn read_cpu(&mut self, addr: u16, _read_only: bool, cartridge: &mut C) -> u8 {
    
        let data = match addr {
            0x0000 => 0x00, // Control
            0x0001 => 0x00, // Mask
            // Status
            0x0002 => {
                let temp = (self.status & 0xE0) | (self.ppu_data_buffer & 0x1F);
                self.status &= !STATUS_VERTICAL_BLANK;
                self.address_latch = 0; 
                temp
            }, 
            // OAM Address - reading from here does not make sense as the CPU does not care about the OAM address
            0x0003 => 0x00,
            // OAM Data
            0x0004 => {
                return self.oam.read(self.oam_addr);
            }, 
            0x0005 => 0x00, // Scroll
            0x0006 => 0x00, // PPU Address
            0x0007 => {
                let temp         = self.ppu_data_buffer;
                let addr        = self.vram_addr.to_u16();
                self.ppu_data_buffer = self.read_ppu(addr, cartridge).unwrap_or(0x00);

                // Auto-increment for convenience - we rarely want to read/write the same location twice
                let new_addr    = addr.wrapping_add(self.ppu_addr_increment());
                self.vram_addr       = Loopy::from_u16(new_addr);

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

    fn write_cpu(&mut self, addr: u16, data: u8, cartridge: &mut C)  {
        match addr {
            // Control
            0x0000 => {
                self.control = data;
                // Set tram_addr.nametable_x/y = control.nametable_x/y
                self.tram_addr.set_nametable_x(((data & CTRL_NAMETABLE_X) != 0) as u8);
                self.tram_addr.set_nametable_y(((data & CTRL_NAMETABLE_Y) != 0) as u8);
            }, 
            // Mask
            0x0001 => {
                self.mask = data;
            }, 
            0x0002 => {}, // Status
            // OAM Address
            0x0003 => {
                self.oam_addr = data;
            }, 
            // OAM Data
            0x0004 => {
                self.oam.write(self.oam_addr, data); 
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }, 
            0x0005 => {
                if self.address_latch == 0 {
                    self.fine_x = data & 0b111;              // first three bits of data
                    self.tram_addr.set_coarse_x(data >> 3);  // bits 4-8
                    self.address_latch = 1;
                } else {
                    self.tram_addr.set_fine_y(data & 0b111); // first three bits of data
                    self.tram_addr.set_coarse_y(data >> 3);  // bits 4-8
                    self.address_latch = 0;
                }
            }, // Scroll
            // PPU Address
            0x0006 => {
                if self.address_latch == 0 {
                    self.tram_addr       = Loopy::from_u16((self.tram_addr.to_u16() & 0x00FF) | ((data as u16) << 8)); 
                    self.address_latch   = 1;
                } else {
                    self.tram_addr       = Loopy::from_u16((self.tram_addr.to_u16() & 0xFF00) | ((data as u16) << 0)); 
                    self.vram_addr       = self.tram_addr;
                    self.address_latch   = 0;
                }
            }, 
            // PPU Data
            0x0007 => {
                let addr = self.vram_addr.to_u16();
                self.write_ppu(addr, data, cartridge);

                let new_addr = addr.wrapping_add(self.ppu_addr_increment());
                
                // Auto-increment for convenience - we rarely want to read/write the same location twice
                self.vram_addr = Loopy::from_u16(new_addr);
            }, 
            _      => {},
        };
    }

    fn read_ppu(&mut self, addr: u16, cartridge: &mut C) -> Option<u8> {
        let mut addr = addr & 0x3FFF;


        if let Some(data) = cartridge.read_ppu(addr) {
            return Some(data);
        } else if addr <= 0x1FFF
        {
            // If the cartridge cant map the address, have
            // a physical location ready here
            let table  = (addr & 0x1000) >> 12; // 0 or 1
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

    fn write_ppu(&mut self, addr: u16, data: u8, cartridge: &mut C) {
        let mut addr = addr & 0x3FFF;


        if let Some(_) = cartridge.write_ppu(addr & 0x3FFF, data) {

        } else if addr <= 0x1FFF
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
            addr &= 0x001F;
            if addr == 0x0010 {addr = 0x0000;}
            if addr == 0x0014 {addr = 0x0004;}
            if addr == 0x0018 {addr = 0x0008;}
            if addr == 0x001C {addr = 0x000C;}
                    
            self.table_palette[addr as usize] = data;
        }
    }

}

impl<C: CartridgeInterface> Olc2c02<C> {

    pub fn get_name_table(&self) -> Vec<u8> {
        self.table_name[..1024].to_vec()
    }

    pub fn get_pattern_table(&mut self, i: u8, palette: u8, cartridge: &mut C) -> Vec<u8> {
        
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
                        // Turns out that we need the addition to get the right output

                        let pixel = (tile_lsb & 0x01) + (tile_msb & 0x01); 
                        tile_lsb >>= 1;
                        tile_msb >>= 1; 

                        let x: u16 = n_tile_x * 8 + (7 - col);
                        let y: u16 = n_tile_y * 8 + row;
                        let w: u16 = 128;
                        
                        let index = self.get_colour_from_palette_ram(palette, pixel, cartridge).unwrap_or(0);

                        sprite_pattern_table[(x + y * w) as usize] = index; 
                    }
                }

            }
        }
        
        sprite_pattern_table.to_vec()
    }

    
	// This is a convenience function that takes a specified palette and pixel
	// index and returns the appropriate screen colour.
    // Originally, javidx9 used read_ppu here but since this is a hot path in the code
    // I decided to move the logic into the function itself
    fn get_colour_from_palette_ram(&self, palette: u8, pixel: u8, _cartridge: &C) -> Option<u8> {
        let mut addr = ((palette as usize) << 2) | (pixel as usize & 0x03);
        addr &= 0x1F;
        if addr == 0x10 { addr = 0x00; }
        if addr == 0x14 { addr = 0x04; }
        if addr == 0x18 { addr = 0x08; }
        if addr == 0x1C { addr = 0x0C; }
        Some(self.table_palette[addr])
    }
}