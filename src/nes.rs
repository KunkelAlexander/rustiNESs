#![allow(dead_code, unused, unused_variables, unused_imports, unused_comparisons)]
use crate::interfaces::BusInterface;
use crate::bus::Bus;
use crate::cpu::Olc6502;
use crate::ppu::Olc2c02;
use crate::cartridge::Cartridge;



// System clock rate (NTSC): CPU 1,789,773 Hz × 3 = 5,369,319 system clocks/sec
const SYSTEM_CLOCK_RATE: u32 = 5_369_319;
const AUDIO_SAMPLE_RATE: u32 = 44_100;
const AUDIO_BUFFER_SIZE: usize = 1024; // enough for one frame (~735 samples)

pub struct Nes {
    cpu: Olc6502<Bus<Cartridge>>,
    bus: Bus<Cartridge>,
    system_clock_counter: u32,
    cpu_divider: u8,
    audio_acc: u32,    // integer audio timing
    audio_buffer: [f32; AUDIO_BUFFER_SIZE],
    audio_buffer_len: usize,
    sine_phase:   f32,
}


impl Nes {
    pub fn new() -> Self {
        Self {
            cpu:                    Olc6502::new(),
            bus:                    Bus::new(Cartridge::new()),
            system_clock_counter:   0,
            cpu_divider:            0,
            audio_acc:              0,
            audio_buffer:           [0.0; AUDIO_BUFFER_SIZE],
            audio_buffer_len:       0,
            sine_phase:             0.0,
        }
    }

    pub fn audio_ptr(&self) -> *const f32 {
        self.audio_buffer.as_ptr()
    }

    pub fn audio_len(&self) -> usize {
        self.audio_buffer_len
    }

    pub fn reset(&mut self) {
        self.bus.reset();
        self.cpu.reset(&mut self.bus);
        self.system_clock_counter = 0;
        self.cpu_divider          = 0;
        self.audio_acc            = 0;
    }

    pub fn cpu_clock(&mut self) {
        self.cpu.clock(&mut self.bus);
    }

    pub fn clock(&mut self) {
        self.bus.clock();

        if self.cpu_divider == 0 {
            self.cpu_divider = 2;
            // Once a DMA transfer is requested, we wait until the correct clock cycle required by the hardware and then start the transfer
            // We read on even cycles and write on odd cycles until we are done
            if self.bus.dma_transfer {
                if self.bus.dma_dummy {
                    if self.system_clock_counter & 1 == 1 {
                        self.bus.dma_dummy = false;
                    }
                }
                else // if self.bus.dma_dummy
                {
                    // On even cycles, read data from the CPU
                    if self.system_clock_counter & 1 == 0 {
                        let addr  = ((self.bus.dma_page as u16) << 8) | self.bus.dma_addr as u16;
                        let data  = self.bus.read(addr, false);
                        self.bus.dma_data = data;
                    }
                    // On odd cycles, write to the PPU's memory
                    else {
                        let addr = self.bus.dma_addr;
                        let data = self.bus.dma_data;
                        self.bus.ppu.oam.write(addr, data);
                        self.bus.dma_addr = self.bus.dma_addr.wrapping_add(1);

                        // We know that the transfer has finished if dma_addr is zero again because it has wrapped around after 256 cycles
                        if self.bus.dma_addr == 0x00 {
                            self.bus.dma_transfer = false;
                            self.bus.dma_dummy    = true;
                        }
                    }
                }
            }
            else // if self.bus.dma_transfer
            {
                self.cpu.clock(&mut self.bus);
            }
        } else {
            self.cpu_divider -= 1;
        }

        if self.bus.ppu.nmi {
            self.bus.ppu.nmi = false;
            self.cpu.nmi(&mut self.bus);
        }

        self.system_clock_counter += 1;
    }

    pub fn run_frame(&mut self) {
        self.audio_buffer_len = 0;

        while !self.bus.ppu.frame_complete {
            self.clock();  // advances APU + PPU + CPU timing

            // Integer accumulator: add AUDIO_SAMPLE_RATE per clock, sample when >= SYSTEM_CLOCK_RATE
            self.audio_acc += AUDIO_SAMPLE_RATE;
            if self.audio_acc >= SYSTEM_CLOCK_RATE {
                self.audio_acc -= SYSTEM_CLOCK_RATE;
                if self.audio_buffer_len < AUDIO_BUFFER_SIZE {
                    self.audio_buffer[self.audio_buffer_len] = self.bus.apu.get_output_sample();
                    self.audio_buffer_len += 1;
                }
            }
        }

        self.bus.ppu.frame_complete = false;
    }

    pub fn insert_cartridge(&mut self, cartridge_data: &[u8]) -> Result<(), String> {
        let cart = Cartridge::from_bytes(cartridge_data)?;
        self.bus.insert_cartridge(cart);
        Ok(())
    }

    pub fn frame_ptr(&self) -> *const u8 {
        self.bus.ppu.get_frame_buffer()
    }

    pub fn frame_len(&self) -> usize {
        self.bus.ppu.get_frame_buffer_len()
    }

    pub fn step_instruction(&mut self) {
        self.cpu.step_instruction(&mut self.bus);
    }

    pub fn load_program(&mut self, bytes: &[u8], offset: u16) {
        for (i, byte) in bytes.iter().enumerate() {
            let addr: u16 = offset.wrapping_add(i as u16);
            self.bus.write(addr, *byte);
        }
        // Set reset vector so CPU starts at offset
        self.bus.write(0xFFFC, (offset & 0x00FF) as u8);
        self.bus.write(0xFFFD, (offset >> 8) as u8);

        self.cpu.reset(&mut self.bus);
    }

    pub fn get_registers(&self) -> Vec<u32> {
        let (a, x, y, stkp, pc, status) = self.cpu.get_registers();

        vec![
            a as u32,
            x as u32,
            y as u32,
            stkp as u32,
            pc as u32,
            status as u32,
        ]
    }

    pub fn get_cpu_state(&self) -> Vec<u32> {
        let (fetched, addr_abs, addr_rel, opcode, cycles) = self.cpu.get_state();

        vec![
            fetched as u32,
            addr_abs as u32,
            addr_rel as u32,
            opcode as u32,
            cycles as u32,
        ]
    }

    pub fn get_ram(&self, start: u16, len: usize) -> Vec<u8> {
        self.bus.get_ram(start, len)
    }


    pub fn get_pattern_table(&self, table: u8, palette: u8) -> Vec<u8> {
        self.bus.get_pattern_table(table, palette)
    }

    pub fn get_name_table(&self) -> Vec<u8> {
        self.bus.get_name_table()
    }

    pub fn set_controller(&mut self, i: usize, x: bool, z: bool, a: bool, s: bool, up: bool, down: bool, left: bool, right: bool) {
        self.bus.set_controller(i, x, z, a, s, up, down, left, right);
    }
}
