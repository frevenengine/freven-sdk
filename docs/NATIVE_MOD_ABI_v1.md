# Native Mod ABI v1

This document defines the in-process native dynamic-library transport ABI for
Freven native mods.

The canonical public guest contract is `freven_guest` as documented in
`GUEST_CONTRACT_v1.md`. Native is a secondary unsafe transport that carries the
same guest negotiation and action semantics over an in-process ptr/len ABI.

This is not the recommended public authoring path. Prefer Wasm with
`freven_guest_sdk` unless you are intentionally doing low-level runtime work on
trusted local code.

## Required exports

A native mod dynamic library must export these symbols:

- `freven_guest_alloc(size: u32) -> u32`
- `freven_guest_dealloc(ptr: u32, len: u32)`
- `freven_guest_negotiate(payload_ptr: u32, payload_len: u32) -> u64`
- `freven_guest_handle_action(payload_ptr: u32, payload_len: u32) -> u64` when
  `action_entrypoint = true`
- `freven_guest_on_start_client(payload_ptr: u32, payload_len: u32) -> u64`
  when `lifecycle.start_client = true`
- `freven_guest_on_start_server(payload_ptr: u32, payload_len: u32) -> u64`
  when `lifecycle.start_server = true`
- `freven_guest_on_tick_client(payload_ptr: u32, payload_len: u32) -> u64`
  when `lifecycle.tick_client = true`
- `freven_guest_on_tick_server(payload_ptr: u32, payload_len: u32) -> u64`
  when `lifecycle.tick_server = true`

## Packed pointer/len format

`freven_guest_negotiate` and `freven_guest_handle_action` return a packed
`(ptr,len)` value:

- `((ptr as u64) << 32) | (len as u64)`

Host decodes:

- `ptr = (packed >> 32) as u32`
- `len = packed as u32`

`ptr/len` refer to process address space memory owned by the native mod.
Host copies bytes directly and then calls `freven_guest_dealloc(ptr, len)`.

## Encoding

Returned bytes are postcard-encoded `freven_guest` contract types:

- `freven_guest_negotiate` takes `NegotiationRequest` and returns `NegotiationResponse`
- `freven_guest_handle_action` takes `ActionInput` and returns `ActionResult`
- lifecycle exports take `StartInput` or `TickInput` and return `LifecycleAck`

`ActionInput` carries `binding_id`, `player_id`, `level_id`, `stream_epoch`,
`action_seq`, `at_input_seq`, and opaque payload bytes. Those fields inside the
postcard payload are the single source of truth for action context.

## Runtime behavior

Runtime validates and enforces:

- negotiation selects `GUEST_CONTRACT_VERSION_1`
- `guest_id` matches the resolved mod id
- non-empty action keys
- no duplicate action keys within one guest description
- no duplicate `binding_id` values within one guest description
- max byte caps for negotiation/result/input payload before copying
- declared action/lifecycle surface exactly matches the exported symbol surface
- side-incompatible lifecycle declarations are rejected during attach

On decode/validation/contract errors, attach fails.
On lifecycle or action-call faults, runtime disables that guest mod for the
current runtime session and later lifecycle/action calls reject. That includes
host-side failure to apply guest-declared world effects after a valid
`ActionResult` returns.

## Safety model

Native mods are UNSAFE by design:

- loaded in-process
- no sandbox isolation
- no CPU timeout enforcement
- full process privileges

Use external guest execution (`execution = "external_guest"`) when process isolation/timeouts are required.
