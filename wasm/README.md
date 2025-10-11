# molwasm: WASM bindings for molcore

This crate exposes a minimal WASM interface for parsing XYZ/EXTXYZ and a simple f32 view for zero-copy interop from JS.

## Exports
- parse_xyz_frame(s: string) -> JsValue
  - Returns a JS object `{ meta, atoms: { nrows, columns, data } }` where `data` is an array-of-columns (Float32 arrays) and booleans are 0/1.
- F32View
  - new(shape: number[])
  - len(): number
  - shape(): number[]
  - ptr(): number (byte offset to use with a Float32Array over wasm memory)
  - write_from(typedArray: Float32Array): void
  - to_js_array(): Float32Array
  - sum(): number
  - sum_via_ndarray(): number

## Testing (Node)

Use wasm-pack to run the tests in Node (recommended):

- Install wasm-pack if not present: https://rustwasm.github.io/wasm-pack/installer/
- Run tests:

wasm-pack test --node wasm

This builds the crate for wasm32 and runs the tests under Node with the wasm-bindgen test harness.

## Example

import init, { parse_xyz_frame, F32View } from './pkg/molwasm.js';

(async () => {
  await init();
  const s = `3\nProperties=species:S:1:pos:R:3\nH 0 0 1\nO 0 1 0\nH 1 0 0\n`;
  const frame = parse_xyz_frame(s);
  console.log(frame.atoms.nrows);

  const view = new F32View([2,3]);
  const mem = new Float32Array(wasmMemory.buffer, view.ptr(), view.len());
  mem.set([1,2,3,4,5,6]);
  console.log(view.sum()); // 21
})();
