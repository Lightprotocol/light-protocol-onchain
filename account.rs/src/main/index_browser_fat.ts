export type { Hasher, HashCreator, AccountLoadOptions, Account as WasmAccount } from "./model.js";
export { WasmHasher, hasSimd as hasWasmSimd } from "./wasm.js";
export { WasmHasher as AccountHasher } from "./wasm.js";

import wasm from "./wasm/account_wasm_bg.wasm";
import wasmSimd from "./wasm-simd/account_wasm_simd_bg.wasm";
import { setWasmInit, setWasmSimdInit } from "./wasm.js";

// @ts-ignore
setWasmInit(() => wasm());
// @ts-ignore
setWasmSimdInit(() => wasmSimd());
