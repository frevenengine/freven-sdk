# WASM ABI v1

This document defines the Freven WASM mod ABI for WP7A.

## Scope

- Supports action registration and action handling only.
- Host runs modules with no WASI and no host imports by default.
- `[capabilities]` in `mod.toml` is enforced by runtime with a strict allowlist.

## Required exports

A module must export these symbols:

- `freven_alloc(size: u32) -> u32`
- `freven_dealloc(ptr: u32, size: u32)`
- `freven_init() -> u64`
- `freven_handle_action(kind: u32, ptr: u32, len: u32) -> u64`
- linear memory export named `memory`

`freven_init` and `freven_handle_action` return packed `(ptr, len)` as:

- `((ptr as u64) << 32) | (len as u64)`

The host copies returned bytes from guest memory and then calls `freven_dealloc(ptr, len)`.

## Encoding

ABI payloads are `postcard` encoded structs from `crates/freven_wasm_abi`.

### Manifest (`freven_init` return bytes)

`ModManifestV1`:

- `abi_version: u32` (must be `1`)
- `actions: Vec<ActionBindingV1>`

`ActionBindingV1`:

- `key: String` (runtime action key, example `freven.example:wasm_set_block`)
- `kind: u32` (module-local dispatch id passed to `freven_handle_action`)

Host behavior:

- validates `abi_version == 1`
- registers each `actions[].key` as runtime action kind
- maps runtime action kind to `actions[].kind` for callback dispatch

### Action input (`freven_handle_action` input bytes)

`ActionInputV1`:

- `player_id: u64`
- `at_input_seq: u32`
- `payload: &[u8]` (opaque client/server action payload)

### Action result (`freven_handle_action` return bytes)

`ActionResultV1`:

- `outcome: ActionOutcomeV1` (`applied` or `rejected`)
- `edits: Vec<WorldEditV1>`

`WorldEditV1` is a `postcard`-encoded Rust enum.

ABI rule: enum variant order is ABI-significant.
- Do NOT reorder variants.
- Do NOT rename variants expecting any effect on binary encoding.
- Only append new variants at the end.

Currently supported variants:

- `SetBlock { pos: (i32, i32, i32), block_id: u8 }`

Host applies `SetBlock` edits through server world-edit APIs. Any decode/trap/apply failure is treated as `Rejected`.

## Capability policy (implemented in `freven_runtime_wasm`)

Runtime accepts only these capability keys:

- `max_call_millis` (integer, must be `> 0`, cannot exceed host policy max)
- `max_linear_memory_bytes` (integer, must be `> 0`, cannot exceed host policy max)
- `allow_unstable` (boolean, must be `false`)

Unknown keys are rejected. Invalid types are rejected.

Current host policy maxima/defaults:

- `max_call_millis`: `25`
- `max_linear_memory_bytes`: `4 MiB`
- `max_manifest_bytes`: `64 KiB`
- `max_result_bytes`: `256 KiB`
- `max_input_payload_bytes`: `64 KiB`
- `max_edits` per action result: `128`

Capabilities may tighten selected limits (`max_call_millis`, `max_linear_memory_bytes`) but cannot raise limits above policy maxima.

## Security defaults (WP7A)

- No WASI.
- No filesystem access.
- No network access.
- No host function imports.

Only required guest exports are invoked.

## Host limits (WP7B+)

Host implementations may enforce runtime resource limits and reject calls that exceed them.
Common limits include:

- maximum call time budget
- maximum linear memory usage
- maximum input payload bytes accepted from runtime to guest
- maximum output bytes for `freven_init` manifest and `freven_handle_action` result

Guest modules must return packed `(ptr, len)` ranges that are valid and within host-configured
size limits. If a call exceeds limits (time, memory, or byte caps), host may reject/trap the call
and runtime treats the action as rejected.

Time budgets may be enforced using Wasmtime epoch deadlines driven by host epoch ticking
(implementation detail only; ABI contract is unchanged).
