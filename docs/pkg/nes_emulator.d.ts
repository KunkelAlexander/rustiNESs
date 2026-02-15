/* tslint:disable */
/* eslint-disable */

export class Emulator {
    free(): void;
    [Symbol.dispose](): void;
    clock(): void;
    get_cpu_state(): Uint32Array;
    get_ram(start: number, len: number): Uint8Array;
    get_registers(): Uint32Array;
    load_program(bytes: Uint8Array, offset: number): void;
    constructor();
    reset(): void;
    step_instruction(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_emulator_free: (a: number, b: number) => void;
    readonly emulator_clock: (a: number) => void;
    readonly emulator_get_cpu_state: (a: number) => [number, number];
    readonly emulator_get_ram: (a: number, b: number, c: number) => [number, number];
    readonly emulator_get_registers: (a: number) => [number, number];
    readonly emulator_load_program: (a: number, b: number, c: number, d: number) => void;
    readonly emulator_new: () => number;
    readonly emulator_reset: (a: number) => void;
    readonly emulator_step_instruction: (a: number) => void;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
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
