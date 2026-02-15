/* tslint:disable */
/* eslint-disable */

export function main(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly main: () => void;
  readonly wasm_bindgen__convert__closures_____invoke__h0d6bdf1e96d76240: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h1e0d64ef7fe765ef: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hfbaa3910296fbfcd: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h29774640a7a17e05: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h850f600ade7a98d2: (a: number, b: number) => void;
  readonly wasm_bindgen__closure__destroy__he7ddba877651b892: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h66571d562505e3d4: (a: number, b: number) => [number, number];
  readonly wasm_bindgen__convert__closures_____invoke__h48b279bce80960e2: (a: number, b: number, c: any, d: any) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
