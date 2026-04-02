/* tslint:disable */
/* eslint-disable */

export class NES {
    free(): void;
    [Symbol.dispose](): void;
    clock(): void;
    cpu_clock(): void;
    frame(): Uint8Array;
    get_cpu_state(): Uint32Array;
    get_pattern_table(table: number, palette: number): Uint8Array;
    get_ram(start: number, len: number): Uint8Array;
    get_registers(): Uint32Array;
    insert_cartridge(cartridge_data: Uint8Array): void;
    load_program(bytes: Uint8Array, offset: number): void;
    constructor();
    reset(): void;
    run_frame(): void;
    step_instruction(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_nes_free: (a: number, b: number) => void;
    readonly nes_clock: (a: number) => void;
    readonly nes_cpu_clock: (a: number) => void;
    readonly nes_frame: (a: number) => [number, number];
    readonly nes_get_cpu_state: (a: number) => [number, number];
    readonly nes_get_pattern_table: (a: number, b: number, c: number) => [number, number];
    readonly nes_get_ram: (a: number, b: number, c: number) => [number, number];
    readonly nes_get_registers: (a: number) => [number, number];
    readonly nes_insert_cartridge: (a: number, b: number, c: number) => [number, number];
    readonly nes_load_program: (a: number, b: number, c: number, d: number) => void;
    readonly nes_new: () => number;
    readonly nes_reset: (a: number) => void;
    readonly nes_run_frame: (a: number) => void;
    readonly nes_step_instruction: (a: number) => void;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
