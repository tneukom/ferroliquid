/* tslint:disable */
/* eslint-disable */

export function main(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly main: () => void;
  readonly wasm_bindgen__convert__closures_____invoke__hbf78ec929c9eb627: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__hb8c223a78e1aba48: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h89dbaa756f05f7da: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h7c57f95c17882a8f: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h9a715c49c5f0c61c: (a: number, b: number) => void;
  readonly wasm_bindgen__closure__destroy__hf22115f63329795c: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h38f768a5af7dcee1: (a: number, b: number) => [number, number];
  readonly wasm_bindgen__convert__closures_____invoke__h064e50732a3c261c: (a: number, b: number, c: any, d: any) => void;
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
