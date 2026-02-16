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

            // Validate final CPU state
            assert_cpu_matches(&cpu, &case.final_state, &format!("{} case {} '{}'", opcode_file, i, case.name));

            // Validate final RAM state (only specified addresses)
            assert_ram_matches(&bus, &case.final_state, &format!("{} case {} '{}'", opcode_file, i, case.name));
        }
    }
}
