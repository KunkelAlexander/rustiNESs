#![allow(dead_code, unused, unused_variables, unused_imports, unused_comparisons)]
pub mod bus;
pub mod cpu;

use wasm_bindgen::prelude::*;
use crate::bus::Bus;
use crate::cpu::Olc6502;

#[wasm_bindgen]
pub struct Emulator {
    cpu: Olc6502,
    bus: Bus,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            cpu: Olc6502::new(),
            bus: Bus::new(),
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.bus);
    }

    pub fn clock(&mut self) {
        self.cpu.clock(&mut self.bus);
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

        self.reset();
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

}
