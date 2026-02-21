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
macro_rules! harte_test {
    ($name:ident, $file:expr) => {
        #[test]
        fn $name() {
            run_opcode_file($file);
        }
    };
}

fn run_opcode_file(filename : &str) {
    let path = Path::new("tests/harte/nes6502/v1").join(filename);


    let mut cpu = Olc6502::new();
    let mut bus = Bus::new();

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


harte_test!(opcode_00, "00.json");
harte_test!(opcode_01, "01.json");
harte_test!(opcode_05, "05.json");
harte_test!(opcode_06, "06.json");
harte_test!(opcode_08, "08.json");
harte_test!(opcode_09, "09.json");
harte_test!(opcode_0a, "0a.json");
harte_test!(opcode_0d, "0d.json");
harte_test!(opcode_0e, "0e.json"); 
harte_test!(opcode_10, "10.json");
harte_test!(opcode_11, "11.json");
harte_test!(opcode_15, "15.json");
harte_test!(opcode_16, "16.json");
harte_test!(opcode_18, "18.json");
harte_test!(opcode_19, "19.json");
harte_test!(opcode_1d, "1d.json");
harte_test!(opcode_1e, "1e.json"); 
harte_test!(opcode_20, "20.json");
harte_test!(opcode_21, "21.json");
harte_test!(opcode_24, "24.json");
harte_test!(opcode_25, "25.json");
harte_test!(opcode_26, "26.json");
harte_test!(opcode_28, "28.json");
harte_test!(opcode_29, "29.json");
harte_test!(opcode_2a, "2a.json");
harte_test!(opcode_2c, "2c.json");
harte_test!(opcode_2d, "2d.json");
harte_test!(opcode_2e, "2e.json"); 
harte_test!(opcode_30, "30.json");
harte_test!(opcode_31, "31.json");
harte_test!(opcode_35, "35.json");
harte_test!(opcode_36, "36.json");
harte_test!(opcode_38, "38.json");
harte_test!(opcode_39, "39.json");
harte_test!(opcode_3d, "3d.json");
harte_test!(opcode_3e, "3e.json"); 
harte_test!(opcode_40, "40.json");
harte_test!(opcode_41, "41.json");
harte_test!(opcode_45, "45.json");
harte_test!(opcode_46, "46.json");
harte_test!(opcode_48, "48.json");
harte_test!(opcode_49, "49.json");
harte_test!(opcode_4a, "4a.json");
harte_test!(opcode_4c, "4c.json");
harte_test!(opcode_4d, "4d.json");
harte_test!(opcode_4e, "4e.json"); 
harte_test!(opcode_50, "50.json");
harte_test!(opcode_51, "51.json");
harte_test!(opcode_55, "55.json");
harte_test!(opcode_56, "56.json");
harte_test!(opcode_58, "58.json");
harte_test!(opcode_59, "59.json");
harte_test!(opcode_5d, "5d.json");
harte_test!(opcode_5e, "5e.json"); 
harte_test!(opcode_60, "60.json");
harte_test!(opcode_61, "61.json");
harte_test!(opcode_65, "65.json");
harte_test!(opcode_66, "66.json");
harte_test!(opcode_68, "68.json");
harte_test!(opcode_69, "69.json");
harte_test!(opcode_6a, "6a.json");
harte_test!(opcode_6c, "6c.json");
harte_test!(opcode_6d, "6d.json");
harte_test!(opcode_6e, "6e.json"); 
harte_test!(opcode_70, "70.json");
harte_test!(opcode_71, "71.json");
harte_test!(opcode_75, "75.json");
harte_test!(opcode_76, "76.json");
harte_test!(opcode_78, "78.json");
harte_test!(opcode_79, "79.json");
harte_test!(opcode_7d, "7d.json");
harte_test!(opcode_7e, "7e.json"); 
harte_test!(opcode_81, "81.json");
harte_test!(opcode_84, "84.json");
harte_test!(opcode_85, "85.json");
harte_test!(opcode_86, "86.json");
harte_test!(opcode_88, "88.json");
harte_test!(opcode_8a, "8a.json");
harte_test!(opcode_8c, "8c.json");
harte_test!(opcode_8d, "8d.json");
harte_test!(opcode_8e, "8e.json");
harte_test!(opcode_90, "90.json");
harte_test!(opcode_91, "91.json");
harte_test!(opcode_94, "94.json");
harte_test!(opcode_95, "95.json");
harte_test!(opcode_96, "96.json");
harte_test!(opcode_98, "98.json");
harte_test!(opcode_99, "99.json");
harte_test!(opcode_9a, "9a.json");
harte_test!(opcode_9d, "9d.json"); 
harte_test!(opcode_a0, "a0.json");
harte_test!(opcode_a1, "a1.json");
harte_test!(opcode_a2, "a2.json");
harte_test!(opcode_a4, "a4.json");
harte_test!(opcode_a5, "a5.json");
harte_test!(opcode_a6, "a6.json");
harte_test!(opcode_a8, "a8.json");
harte_test!(opcode_a9, "a9.json");
harte_test!(opcode_aa, "aa.json");
harte_test!(opcode_ac, "ac.json");
harte_test!(opcode_ad, "ad.json");
harte_test!(opcode_ae, "ae.json");
harte_test!(opcode_b0, "b0.json");
harte_test!(opcode_b1, "b1.json");
harte_test!(opcode_b4, "b4.json");
harte_test!(opcode_b5, "b5.json");
harte_test!(opcode_b6, "b6.json");
harte_test!(opcode_b8, "b8.json");
harte_test!(opcode_b9, "b9.json");
harte_test!(opcode_ba, "ba.json");
harte_test!(opcode_bc, "bc.json");
harte_test!(opcode_bd, "bd.json");
harte_test!(opcode_be, "be.json"); 
harte_test!(opcode_c0, "c0.json");
harte_test!(opcode_c1, "c1.json");
harte_test!(opcode_c4, "c4.json");
harte_test!(opcode_c5, "c5.json");
harte_test!(opcode_c6, "c6.json");
harte_test!(opcode_c8, "c8.json");
harte_test!(opcode_c9, "c9.json");
harte_test!(opcode_ca, "ca.json");
harte_test!(opcode_cc, "cc.json");
harte_test!(opcode_cd, "cd.json");
harte_test!(opcode_ce, "ce.json");
harte_test!(opcode_d0, "d0.json");
harte_test!(opcode_d1, "d1.json");
harte_test!(opcode_d5, "d5.json");
harte_test!(opcode_d6, "d6.json");
harte_test!(opcode_d8, "d8.json");
harte_test!(opcode_d9, "d9.json");
harte_test!(opcode_dd, "dd.json");
harte_test!(opcode_de, "de.json");
harte_test!(opcode_e0, "e0.json");
harte_test!(opcode_e1, "e1.json");
harte_test!(opcode_e4, "e4.json");
harte_test!(opcode_e5, "e5.json");
harte_test!(opcode_e6, "e6.json");
harte_test!(opcode_e8, "e8.json");
harte_test!(opcode_e9, "e9.json");
harte_test!(opcode_ea, "ea.json");
harte_test!(opcode_ec, "ec.json");
harte_test!(opcode_ed, "ed.json");
harte_test!(opcode_ee, "ee.json"); 
harte_test!(opcode_f0, "f0.json");
harte_test!(opcode_f1, "f1.json");
harte_test!(opcode_f5, "f5.json");
harte_test!(opcode_f6, "f6.json");
harte_test!(opcode_f8, "f8.json");
harte_test!(opcode_f9, "f9.json");
harte_test!(opcode_fd, "fd.json");
harte_test!(opcode_fe, "fe.json");