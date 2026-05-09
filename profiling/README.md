# Approach for profiling

- Build CLI emulator, run for 10,000 frames, print average time per frame without GUI and profile with samply. The output json files can be opened in the Firefox profiler, the hashes in the filenames refer to git commit hashes and the times refer to ms per frame. There is some variation in the performance per frame, however. Runs 3 and 4 may actually be the same. 


## Code

`main.rs`:

```rust
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
use std::io::{Write, BufWriter};


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
```

`Cargo.toml`:

```toml
[package]
name = "nes_emulator"

version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[[bin]]
name = "nes_cli"
path = "src/main.rs"


[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[profile.release]
debug = true
strip = false
lto = false
codegen-units = 1
panic = "abort"
```


## Usage

Build 

```bash
RUSTFLAGS="-C force-frame-pointers=yes" cargo build --release
```

Why:

`debug = true` keeps symbols so Firefox Profiler can show function names.

`strip = false` keeps those symbols in the binary.

`force-frame-pointers=yes` improves stack unwinding reliability.

`lto = false` avoids excessive inlining/cross-crate merging that can make profiles harder to interpret. You can later compare with `lto = "thin"` if that matches production.

`codegen-units = 1` gives more optimized, stable codegen, though builds are slower.

With `samply`, save to a file:

```powershell
samply record --save-only --output profile.json .\target\release\nes_cli.exe
```

Later, open it at:

```text
https://profiler.firefox.com/
```

Then drag-and-drop `profile.json` into the page.

