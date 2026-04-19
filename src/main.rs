pub mod bus;
pub mod cpu;
pub mod interfaces;
pub mod ppu;
pub mod cartridge;
pub mod mapper;
pub mod nes;

pub use nes::Nes;

use std::fs;
use std::io::{Write, BufWriter};
use std::error::Error;


fn output_pattern_table(emu: &Nes, path: &str) -> std::io::Result<()> {
    //get pattern table (table 0, palette 0 for example)
    let pattern = emu.get_pattern_table(0, 0);
    
    println!("Pattern table generated: {} bytes", pattern.len());
    
    let width = 128;
    let height = 128;
    
    assert_eq!(pattern.len(), width * height);
    
    let file = fs::File::create(path)?;
    let mut writer = BufWriter::new(file);
    
    for y in 0..height {
        for x in 0..width {
            let val = pattern[y * width + x];
            write!(writer, "{:3} ", val)?; // padded for alignment
        }
        writeln!(writer)?;
    }
    
    println!("Wrote {}", path);
    
    Ok(())
}


fn output_name_table(emu: &Nes, path: &str) -> std::io::Result<()> {
    let name_table = emu.get_name_table();

    println!("Name table generated: {} bytes", name_table.len());
    assert_eq!(name_table.len(), 1024);

    let file = fs::File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "=== NAMETABLE DUMP ===")?;
    writeln!(writer, "Total bytes: {}", name_table.len())?;
    writeln!(writer)?;

    // First 960 bytes: tile IDs, arranged as 32x30
    writeln!(writer, "--- Tile indices (32x30) ---")?;
    for y in 0..30 {
        for x in 0..32 {
            let idx = y * 32 + x;
            let val = name_table[idx];
            write!(writer, "{:02X} ", val)?;
        }
        writeln!(writer)?;
    }

    writeln!(writer)?;
    writeln!(writer, "--- Attribute table (8x8 bytes) ---")?;

    // Last 64 bytes: attribute table
    for y in 0..8 {
        for x in 0..8 {
            let idx = 960 + y * 8 + x;
            let val = name_table[idx];
            write!(writer, "{:02X} ", val)?;
        }
        writeln!(writer)?;
    }

    println!("Wrote {}", path);
    Ok(())
}

fn output_frame(emu: &Nes, path: &str) -> Result<(), Box<dyn Error>> {
    let frame = emu.frame();
    let width = 256;
    let height = 240;

    let charset = [' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

    let scale_x = 4;
    let scale_y = 4;

    let file = fs::File::create(path)?;
    let mut writer = BufWriter::new(file);

    for y in (0..height).step_by(scale_y) {
        for x in (0..width).step_by(scale_x) {
            let val = frame[y * width + x] as usize;

            let idx = val * (charset.len() - 1) / 63;
            write!(writer, "{}", charset[idx])?;
        }
        writeln!(writer)?; // newline
    }

    writer.flush()?; // optional but nice

    println!("Saved ASCII frame to {}", path);
    Ok(())
}


fn main() -> std::io::Result<()> {
    // adjust this path to your Downloads folder
    let rom_path = r"roms/dk.nes";

    // read file into bytes
    let bytes = fs::read(rom_path).expect("failed to read ROM");

    // create emulator
    let mut emu = Nes::new();

    // load ROM
    emu.insert_cartridge(&bytes).expect("failed to load ROM");
    emu.reset();

    
    println!("Loaded ROM");

    // Dump before running
    output_pattern_table(&emu, "output/pattern_table_before.txt")?;
    output_name_table   (&emu, "output/name_table_before.txt")?;
    output_frame        (&emu, "output/frame_before.txt");

    for frame in 0..100 {
        emu.run_frame();
    }
    
    // Dump after running
    output_pattern_table(&emu, "output/pattern_table_after.txt")?;
    output_name_table   (&emu, "output/name_table_after.txt")?;
    output_frame        (&emu, "output/frame_after.txt");

    Ok(())
}