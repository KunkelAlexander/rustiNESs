pub mod bus;
pub mod cpu;
pub mod interfaces;
pub mod ppu;
pub mod apu;
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

    pub fn frame_ptr(&self) -> *const u8 {
        self.inner.frame_ptr()
    }

    pub fn frame_len(&self) -> usize {
        self.inner.frame_len()
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

    pub fn get_pattern_table(&mut self, table: u8, palette: u8) -> Vec<u8> {
        self.inner.get_pattern_table(table, palette)
    }

    pub fn set_controller(&mut self, i: usize, x: bool, z: bool, a: bool, s: bool, up: bool, down: bool, left: bool, right: bool) {
        self.inner
            .set_controller(i, x, z, a, s, up, down, left, right);
    }

    
    pub fn audio_ptr(&self) -> *const f32 {
        self.inner.audio_ptr()
    }

    pub fn audio_len(&self) -> usize {
        self.inner.audio_len()
    }

    pub fn save_state(&self) -> Vec<u8> {
        self.inner.save_state_binary()
    }

    pub fn load_state(&mut self, data: &[u8]) -> Result<(), String> {
        self.inner.load_state_binary(data)
    }

    pub fn save_state_json(&self) -> Vec<u8> {
        self.inner.save_state_json()
    }

    pub fn load_state_json(&mut self, data: &[u8]) -> Result<(), String> {
        self.inner.load_state_json(data)
    }
}