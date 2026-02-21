use std::fs;
use std::path::Path;

use serde::Deserialize;

use nes_emulator::bus::Bus;
use nes_emulator::cpu::Olc6502;

//
// JSON structs
//

#[derive(Debug, Deserialize)]
struct HarteCase {
    name: String,
    initial: HarteState,
    #[serde(rename = "final")]
    final_state: HarteState,
    cycles: Vec<(u16, u8, String)>,
}

#[derive(Debug, Deserialize)]
struct HarteState {
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    ram: Vec<(u16, u8)>,
}

//
// Helpers
//

fn load_cases_from_file(path: &Path) -> Vec<HarteCase> {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

    serde_json::from_str::<Vec<HarteCase>>(&text)
        .unwrap_or_else(|e| panic!("Failed to parse JSON {}: {}", path.display(), e))
}

fn init_bus_from_state(bus: &mut Bus, state: &HarteState) {
    // Clear RAM
    // If your Bus has no "clear", brute force it:
    for addr in 0u16..=0xFFFF {
        bus.write(addr, 0);
    }

    // Apply RAM patches
    for (addr, val) in &state.ram {
        bus.write(*addr, *val);
    }
}

fn set_cpu_from_state(cpu: &mut Olc6502, state: &HarteState) {
    // You said you added this.
    cpu.set_registers(
        state.a,
        state.x,
        state.y,
        state.s,
        state.pc,
        state.p,
    );

    // IMPORTANT:
    // Harte assumes we're starting at an instruction boundary.
    // Many OLC-style cores have an internal "cycles" field.
    // If you have something like cpu.cycles = 0, do it here.
    //
    // If you don't have it exposed, you can add a test-only setter.
    cpu.force_cycles_zero();
}

fn run_one_instruction(cpu: &mut Olc6502, bus: &mut Bus) -> usize {
    // Run cycles until the instruction finishes.
    // The most robust approach:
    // - tick once (starts instruction)
    // - then tick until cpu says it's done

    let mut cycles = 0usize;

    cpu.clock(bus);
    cycles += 1;

    while cpu.get_remaining_cycles() > 0 {
        cpu.clock(bus);
        cycles += 1;
    }

    cycles
}

fn assert_cpu_matches(cpu: &Olc6502, expected: &HarteState, case_name: &str) {
    let (a, x, y, s, pc, p) = cpu.get_registers();

    assert_eq!(
        pc, expected.pc,
        "[{}] PC mismatch: got {:04X}, expected {:04X}",
        case_name, pc, expected.pc
    );
    assert_eq!(
        s, expected.s,
        "[{}] S mismatch: got {:02X}, expected {:02X}",
        case_name, s, expected.s
    );
    assert_eq!(
        a, expected.a,
        "[{}] A mismatch: got {:02X}, expected {:02X}",
        case_name, a, expected.a
    );
    assert_eq!(
        x, expected.x,
        "[{}] X mismatch: got {:02X}, expected {:02X}",
        case_name, x, expected.x
    );
    assert_eq!(
        y, expected.y,
        "[{}] Y mismatch: got {:02X}, expected {:02X}",
        case_name, y, expected.y
    );
    assert_eq!(
        p, expected.p,
        "[{}] P mismatch: got {:02X}, expected {:02X}",
        case_name, p, expected.p
    );
}

fn assert_ram_matches(bus: &Bus, expected: &HarteState, case_name: &str) {
    for (addr, expected_val) in &expected.ram {
        let got = bus.read(*addr, true);
        assert_eq!(
            got, *expected_val,
            "[{}] RAM mismatch at {:04X}: got {:02X}, expected {:02X}",
            case_name, addr, got, expected_val
        );
    }
}

fn is_official_opcode(op: u8) -> bool {
    matches!(op,
        0x00 | 0x01 | 0x05 | 0x06 | 0x08 | 0x09 | 0x0A | 0x0D | 0x0E |
        0x10 | 0x11 | 0x15 | 0x16 | 0x18 | 0x19 | 0x1D | 0x1E |
        0x20 | 0x21 | 0x24 | 0x25 | 0x26 | 0x28 | 0x29 | 0x2A | 0x2C | 0x2D | 0x2E |
        0x30 | 0x31 | 0x35 | 0x36 | 0x38 | 0x39 | 0x3D | 0x3E |
        0x40 | 0x41 | 0x45 | 0x46 | 0x48 | 0x49 | 0x4A | 0x4C | 0x4D | 0x4E |
        0x50 | 0x51 | 0x55 | 0x56 | 0x58 | 0x59 | 0x5D | 0x5E |
        0x60 | 0x61 | 0x65 | 0x66 | 0x68 | 0x69 | 0x6A | 0x6C | 0x6D | 0x6E |
        0x70 | 0x71 | 0x75 | 0x76 | 0x78 | 0x79 | 0x7D | 0x7E |
        0x81 | 0x84 | 0x85 | 0x86 | 0x88 | 0x8A | 0x8C | 0x8D | 0x8E |
        0x90 | 0x91 | 0x94 | 0x95 | 0x96 | 0x98 | 0x99 | 0x9A | 0x9D |
        0xA0 | 0xA1 | 0xA2 | 0xA4 | 0xA5 | 0xA6 | 0xA8 | 0xA9 | 0xAA | 0xAC | 0xAD | 0xAE |
        0xB0 | 0xB1 | 0xB4 | 0xB5 | 0xB6 | 0xB8 | 0xB9 | 0xBA | 0xBC | 0xBD | 0xBE |
        0xC0 | 0xC1 | 0xC4 | 0xC5 | 0xC6 | 0xC8 | 0xC9 | 0xCA | 0xCC | 0xCD | 0xCE |
        0xD0 | 0xD1 | 0xD5 | 0xD6 | 0xD8 | 0xD9 | 0xDD | 0xDE |
        0xE0 | 0xE1 | 0xE4 | 0xE5 | 0xE6 | 0xE8 | 0xE9 | 0xEA | 0xEC | 0xED | 0xEE |
        0xF0 | 0xF1 | 0xF5 | 0xF6 | 0xF8 | 0xF9 | 0xFD | 0xFE
    )
}

//
// Main test
//

#[test]
fn harte_nes6502_v1_all_opcodes() {
    let base_dir = Path::new("tests/harte/nes6502/v1");

    assert!(
        base_dir.exists(),
        "Harte tests not found at {}. Put the JSON files there.",
        base_dir.display()
    );

    let mut entries: Vec<_> = fs::read_dir(base_dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().map(|e| e == "json").unwrap_or(false))
        .collect();

    entries.sort();

    assert!(
        !entries.is_empty(),
        "No .json files found in {}",
        base_dir.display()
    );

    let mut cpu = Olc6502::new();
    let mut bus = Bus::new();

    // Run every opcode file
    for path in entries {
        let opcode_file = path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let cases = load_cases_from_file(&path);

        // Sanity: Harte files are typically 10,000 cases
        assert!(
            !cases.is_empty(),
            "No test cases in {}",
            opcode_file
        );

        // Skip illegal opcodes for now
        let opcode: u8 = u8::from_str_radix(&opcode_file[..2], 16).unwrap();

        if !is_official_opcode(opcode) {
            continue;
        }

        println!("Running {}", opcode_file);
        for (i, case) in cases.iter().enumerate() {

            // Setup
            init_bus_from_state(&mut bus, &case.initial);
            set_cpu_from_state(&mut cpu, &case.initial);

            // Run exactly one instruction
            let cycles_taken = run_one_instruction(&mut cpu, &mut bus);

            // Validate cycle count
            let expected_cycles = case.cycles.len();
            assert_eq!(
                cycles_taken, expected_cycles,
                "[{} case {} '{}'] cycle count mismatch: got {}, expected {}",
                opcode_file, i, case.name, cycles_taken, expected_cycles
            );

            let (a, x, y, s, pc, p) = cpu.get_registers();
            
            let expected = &case.final_state;

            let mismatch =
                pc != expected.pc ||
                s  != expected.s  ||
                a  != expected.a  ||
                x  != expected.x  ||
                y  != expected.y  ||
                p  != expected.p;

                if mismatch {
                    println!("Running {} case {}", opcode_file, i);
                    println!(
                        "EXPECTED: PC={:04X} S={:02X} A={:02X} X={:02X} Y={:02X} P={:02X}",
                        expected.pc,
                        expected.s,
                        expected.a,
                        expected.x,
                        expected.y,
                        expected.p,
                    );

                    println!(
                        "GOT:      PC={:04X} S={:02X} A={:02X} X={:02X} Y={:02X} P={:02X}",
                        pc, s, a, x, y, p
                    );
                }

            // Validate final CPU state
            assert_cpu_matches(&cpu, &case.final_state, &format!("{} case {} '{}'", opcode_file, i, case.name));

            // Validate final RAM state (only specified addresses)
            assert_ram_matches(&bus, &case.final_state, &format!("{} case {} '{}'", opcode_file, i, case.name));
        }
    }
}
