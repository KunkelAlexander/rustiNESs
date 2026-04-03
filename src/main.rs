#![allow(dead_code, unused, unused_variables, unused_imports, unused_comparisons)]

#![allow(dead_code, unused, unused_variables, unused_imports, unused_comparisons)]
pub mod bus;
pub mod cpu;
pub mod interfaces;
pub mod ppu; 
pub mod cartridge; 
pub mod mapper; 

use wasm_bindgen::prelude::*;
use crate::interfaces::BusInterface;
use crate::bus::Bus;
use crate::cpu::Olc6502;
use crate::ppu::Olc2c02;
use crate::cartridge::{EmptyCartridge, Cartridge};
use std::fs;

pub struct NES {
    cpu: Olc6502,
    bus: Bus,

    system_clock_counter: u32,
}

impl NES {
    pub fn new() -> Self {
        Self {
            cpu:                  Olc6502::new(),
            bus:                  Bus::new(Box::new(EmptyCartridge)),
            system_clock_counter: 0
        }
    }

    pub fn reset(&mut self) {
        self.bus.reset();
        self.cpu.reset(&mut self.bus);
        self.system_clock_counter = 0; 
    }

    pub fn cpu_clock(&mut self) {
        self.cpu.clock(&mut self.bus);
    }

    pub fn clock(&mut self) {
        self.bus.ppu.clock();

        if self.system_clock_counter % 3 == 0 {
            self.cpu.clock(&mut self.bus);
        }

        self.system_clock_counter += 1;
    }

    pub fn run_frame(&mut self) {
        while !self.bus.ppu.frame_complete {
            self.clock();  // advances PPU + CPU timing
        }

        self.bus.ppu.frame_complete = false;
    }

    pub fn insert_cartridge(&mut self, cartridge_data: &[u8]) -> Result<(), String> {
        let cart = Cartridge::from_bytes(cartridge_data)?;
        self.bus.insert_cartridge(Box::new(cart));
        Ok(())
    }

    pub fn frame(&self) -> Vec<u8> {
        self.bus.ppu.get_frame_buffer()
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
        let (fetched, addr_abs, addr_rel, opcode, cycles)= self.cpu.get_state();

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
}

fn main() {
    // adjust this path to your Downloads folder
    let rom_path = r"C:\Users\TE\Downloads\full_nes_palette.nes";

    // read file into bytes
    let bytes = fs::read(rom_path).expect("failed to read ROM");

    // create emulator
    let mut emu = NES::new();

    // load ROM
    emu.insert_cartridge(&bytes).expect("failed to load ROM");

    
    println!("Loaded ROM");

    // get pattern table (table 0, palette 0 for example)
    let pattern = emu.get_pattern_table(0, 0);

    
    println!("Pattern table generated: {} bytes", pattern.len());
    
    // run some cycles
    for _ in 0..100_000 {
        emu.clock();
    }

    // get pattern table (table 0, palette 0 for example)
    let pattern = emu.get_pattern_table(0, 0);

    println!("Pattern table generated: {} bytes", pattern.len());
    
    // get pattern table (table 0, palette 0 for example)
    let pattern = emu.get_pattern_table(1, 0);

    println!("Pattern table generated: {} bytes", pattern.len());
}
