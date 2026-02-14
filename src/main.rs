#![allow(dead_code, unused, unused_variables, unused_imports, unused_comparisons)]
pub mod bus;
pub mod cpu;


use crate::bus::Bus;
use crate::cpu::Olc6502;

pub struct Nes {
    pub cpu: Olc6502,
    pub bus: Bus,
}

impl Nes {
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
}

fn main() {
    println!("NES emulator starting…");

    let mut nes = Nes::new();
    nes.reset();

    for _ in 0..10 {
        nes.clock();
    }
}
