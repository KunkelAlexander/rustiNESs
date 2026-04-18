#![allow(dead_code, unused, unused_variables, unused_imports, unused_comparisons)]

pub mod bus;
pub mod cpu;
pub mod interfaces;
pub mod ppu;
pub mod cartridge;
pub mod mapper;
pub mod nes;

pub use nes::Nes;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct NES {
    inner: Nes,
}

#[wasm_bindgen]
impl NES {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { inner: Nes::new() }
    }

    pub fn reset(&mut self) {
        self.inner.reset();
    }

    pub fn cpu_clock(&mut self) {
        self.inner.cpu_clock();
    }

    pub fn clock(&mut self) {
        self.inner.clock();
    }

    pub fn run_frame(&mut self) {
        self.inner.run_frame();
    }

    pub fn insert_cartridge(&mut self, cartridge_data: &[u8]) -> Result<(), String> {
        self.inner.insert_cartridge(cartridge_data)
    }

    pub fn frame(&self) -> Vec<u8> {
        self.inner.frame()
    }

    pub fn step_instruction(&mut self) {
        self.inner.step_instruction();
    }

    pub fn load_program(&mut self, bytes: &[u8], offset: u16) {
        self.inner.load_program(bytes, offset);
    }

    pub fn get_registers(&self) -> Vec<u32> {
        self.inner.get_registers()
    }

    pub fn get_cpu_state(&self) -> Vec<u32> {
        self.inner.get_cpu_state()
    }

    pub fn get_ram(&self, start: u16, len: usize) -> Vec<u8> {
        self.inner.get_ram(start, len)
    }

    pub fn get_pattern_table(&self, table: u8, palette: u8) -> Vec<u8> {
        self.inner.get_pattern_table(table, palette)
    }

    pub fn set_controller(
        &mut self,
        i: usize,
        x: bool,
        z: bool,
        a: bool,
        s: bool,
        up: bool,
        down: bool,
        left: bool,
        right: bool,
    ) {
        self.inner
            .set_controller(i, x, z, a, s, up, down, left, right);
    }
}