/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export const __wbg_nes_free: (a: number, b: number) => void;
export const nes_clock: (a: number) => void;
export const nes_cpu_clock: (a: number) => void;
export const nes_frame: (a: number) => [number, number];
export const nes_get_cpu_state: (a: number) => [number, number];
export const nes_get_pattern_table: (a: number, b: number, c: number) => [number, number];
export const nes_get_ram: (a: number, b: number, c: number) => [number, number];
export const nes_get_registers: (a: number) => [number, number];
export const nes_insert_cartridge: (a: number, b: number, c: number) => [number, number];
export const nes_load_program: (a: number, b: number, c: number, d: number) => void;
export const nes_new: () => number;
export const nes_reset: (a: number) => void;
export const nes_run_frame: (a: number) => void;
export const nes_step_instruction: (a: number) => void;
export const __wbindgen_externrefs: WebAssembly.Table;
export const __wbindgen_free: (a: number, b: number, c: number) => void;
export const __wbindgen_malloc: (a: number, b: number) => number;
export const __externref_table_dealloc: (a: number) => void;
export const __wbindgen_start: () => void;
