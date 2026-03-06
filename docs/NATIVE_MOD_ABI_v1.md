# Native Mod ABI v1

This document defines the in-process native dynamic-library ABI for Freven native mods.

This is a legacy action-only transport ABI. The canonical public guest contract
is `freven_guest` as documented in `GUEST_CONTRACT_v1.md`.

## Required exports

A native mod dynamic library must export these symbols:

- `freven_alloc(size: u32) -> u32`
- `freven_dealloc(ptr: u32, len: u32)`
- `freven_init() -> u64`
- `freven_handle_action(kind: u32, payload_ptr: u32, payload_len: u32) -> u64`

## Packed pointer/len format

`freven_init` and `freven_handle_action` return a packed `(ptr,len)` value:

- `((ptr as u64) << 32) | (len as u64)`

Host decodes:

- `ptr = (packed >> 32) as u32`
- `len = packed as u32`

`ptr/len` refer to process address space memory owned by the native mod.
Host copies bytes directly and then calls `freven_dealloc(ptr, len)`.

## Encoding

Returned bytes are postcard-encoded Rust ABI types from `crates/freven_wasm_abi`:

- `freven_init` -> `ModManifestV1`
- `freven_handle_action` -> `ActionResultV1`

Action input bytes passed to `freven_handle_action` are postcard-encoded `ActionInputV1`.

`ActionInputV1` carries `player_id` and `at_input_seq`; these fields inside the postcard payload are
the single source of truth for action context.

## Runtime behavior

Runtime validates and enforces:

- `manifest.abi_version == 1`
- non-empty action keys
- no duplicate action keys within a mod manifest
- no duplicate host `kind` values within a mod manifest
- max byte caps for manifest/result/input payload before copying

On decode/validation/ABI errors, attach fails.
On action-call failures, runtime treats the action as rejected.

## Safety model

Native mods are UNSAFE by design:

- loaded in-process
- no sandbox isolation
- no CPU timeout enforcement
- full process privileges

Use external mods (`kind = "external"`) when process isolation/timeouts are required.
