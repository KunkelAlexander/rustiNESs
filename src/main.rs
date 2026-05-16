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
    let rom_path = r"roms/smb.nes";

    // read file into bytes
    let bytes = fs::read(rom_path).expect("failed to read ROM");

    // create emulator
    let mut emu = Nes::new();

    // load ROM
    emu.insert_cartridge(&bytes).expect("failed to load ROM");
    emu.reset();


    use std::time::Instant;

    //let t0 = Instant::now();
    //for _ in 0..10000 {
    //    emu.run_frame();
    //}
    //let avg = t0.elapsed().as_secs_f64() * 1000.0 / 10000.0;
    //println!("avg over 10,000 frames: {:.2}ms", avg);

    // Save/load round-trip test at frame 100
    let mut emu2 = Nes::new();
    emu2.insert_cartridge(&bytes).expect("failed to load ROM");
    emu2.reset();
    for _ in 0..100 {
        emu2.run_frame();
    }
    let state = emu2.save_state_binary();
    println!("save_state: {} bytes", state.len());
    emu2.load_state_binary(&state).expect("load_state failed");
    for _ in 0..100 {
        emu2.run_frame();
    }
    println!("save/load round-trip: ok");
    
    Ok(())
}