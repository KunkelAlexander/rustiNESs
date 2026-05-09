pub mod bus;
pub mod cpu;
pub mod interfaces;
pub mod ppu;
pub mod apu;
pub mod cartridge;
pub mod mapper;
pub mod nes;

pub use nes::Nes;

use std::fs;

fn main() -> std::io::Result<()> {
    let rom_path = r"roms/dk.nes";

    // read file into bytes
    let bytes = fs::read(rom_path).expect("failed to read ROM");

    // create emulator
    let mut emu = Nes::new();

    // load ROM
    emu.insert_cartridge(&bytes).expect("failed to load ROM");
    emu.reset();


    use std::time::Instant;

    let t0 = Instant::now();
    for _ in 0..10000 {
        emu.run_frame();
    }
    let avg = t0.elapsed().as_secs_f64() * 1000.0 / 10000.0;
    println!("avg over 10,000 frames: {:.2}ms", avg);
    
    Ok(())
}