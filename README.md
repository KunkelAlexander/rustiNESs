# NES Emulator (Rust)

A simple Nintendo Entertainment System (NES) emulator written in **Rust**, built for learning and experimentation.

This project aims to incrementally emulate the original NES hardware, starting with the **6502 CPU** and gradually adding the PPU, APU, cartridge mappers, and input devices.


## Devlog

- 01.02.2026
    - Watch [NES Emulator Part #1: Bitwise Basics & Overview](https://www.youtube.com/watch?v=F8kx56OZQhg)
    - Watch [NES Emulator Part #2: The CPU (6502 Implementation)](https://www.youtube.com/watch?v=8XmxKPJDGU0)

![](figures/1.png)
    - A CPU in isolation does nothing
    - CPU (6502) needs to be conntected to a BUS

![](figures/2.png)

    - Address and data lines of the CPU are connected to the CPU 
    - CPU sets address of the BUS - other devices need to react 
    - BUS has a 16-bit address space from 0x0000 to 0xFFFF
    - Every device gets assigned an address range on the BUS 
    - In our system: 64 kB of RAM containing variables as well as the program itself
    - The CPU extracts bytes from the RAM in order to execute them 
    - We need a CPU, a BUS and a RAM


![](figures/3.png)

    - 16 address bits: A0-A15
    - 8 data bits: D0-D7


![](figures/4.png)
    - Not all instructions are the same length
    - Different instructions need different numbers of clock cycles to execute
    - 56 legal instructions
    - First byte of the instruction 

![](figures/5.png)
    - 

## Goals

- Learn low-level hardware emulation
- Learn Rust
- Build a reasonably accurate (but readable) NES emulator
- Keep the architecture modular and testable

## Hardware Overview

The NES consists of:
- **CPU**: Ricoh RP2A03 (6502-compatible, no decimal mode)
- **PPU**: Picture Processing Unit (graphics)
- **APU**: Audio Processing Unit
- **Cartridge**: PRG-ROM, CHR-ROM, mapper logic
- **Controllers**

The CPU communicates with all components via a shared **address bus**, **data bus**, and **read/write control**.


## Project Structure

```
src/
├── main.rs    # Entry point
├── cpu.rs     # 6502 core (registers, execution)
```

## CPU (6502)

- 8-bit data bus
- 16-bit address bus
- Little-endian
- No BCD (decimal) mode on the NES variant
- Memory-mapped I/O

Registers:
- A (Accumulator)
- X, Y (Index registers)
- PC (Program Counter)
- SP (Stack Pointer)
- P (Status Flags)


## Current Status

- [ ] CPU registers
- [ ] Memory bus
- [ ] Instruction fetch/decode/execute
- [ ] Cycle accuracy
- [ ] PPU
- [ ] APU
- [ ] Input
- [ ] Mapper support


## License

MIT