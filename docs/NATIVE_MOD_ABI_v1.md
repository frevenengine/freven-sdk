# Native Mod ABI v1

This document defines the in-process native dynamic-library transport ABI for
Freven native mods.

The canonical public guest contract is `freven_guest` as documented in
`GUEST_CONTRACT_v1.md`. Native is a secondary unsafe transport that carries the
same guest negotiation, declaration, and callback semantics over an in-process native-width ABI.

This is not the recommended public authoring path. Prefer Wasm with
`freven_guest_sdk` unless you are intentionally doing low-level runtime work on
trusted local code.

## Required exports

A native mod dynamic library must export these symbols:

- `freven_guest_alloc(size: usize) -> *mut u8`
- `freven_guest_dealloc(buffer: NativeGuestBuffer)`
- `freven_guest_negotiate(input: NativeGuestInput) -> NativeGuestBuffer`
- `freven_guest_handle_action(input: NativeGuestInput) -> NativeGuestBuffer` when
  `callbacks.action = true`
- `freven_guest_on_server_messages(input: NativeGuestInput) -> NativeGuestBuffer`
  when `callbacks.server_messages = true`
- `freven_guest_on_start_client(input: NativeGuestInput) -> NativeGuestBuffer`
  when `callbacks.lifecycle.start_client = true`
- `freven_guest_on_start_server(input: NativeGuestInput) -> NativeGuestBuffer`
  when `callbacks.lifecycle.start_server = true`
- `freven_guest_on_tick_client(input: NativeGuestInput) -> NativeGuestBuffer`
  when `callbacks.lifecycle.tick_client = true`
- `freven_guest_on_tick_server(input: NativeGuestInput) -> NativeGuestBuffer`
  when `callbacks.lifecycle.tick_server = true`

FFI structs:

```rust
#[repr(C)]
struct NativeGuestInput {
    ptr: *const u8,
    len: usize,
}

#[repr(C)]
struct NativeGuestBuffer {
    ptr: *mut u8,
    len: usize,
}
```

`usize` tracks the platform-native pointer width. On 64-bit targets the ABI is
64-bit-safe; on 32-bit targets it naturally narrows with the target ABI.

## Memory contract

- Host-to-guest input:
  - non-empty input: host allocates guest-owned memory with `freven_guest_alloc`
  - non-empty input: host copies input bytes into that allocation
  - host calls guest entrypoints with `NativeGuestInput { ptr, len }`
  - non-empty input: host frees the input allocation with `freven_guest_dealloc`
  - empty input is passed canonically as `ptr = null` with `len = 0`
- Guest-to-host output:
  - guest returns `NativeGuestBuffer { ptr, len }`
  - buffer refers to process memory owned by the native mod
  - host copies bytes directly
  - host frees the returned buffer with `freven_guest_dealloc`
- Zero-length buffers must use `ptr = null` with `len = 0`
- Native does not use Wasm-style packed `(ptr,len)` integers anywhere

## Encoding

Returned bytes are postcard-encoded `freven_guest` contract types:

- `freven_guest_negotiate` takes `NegotiationRequest` and returns `NegotiationResponse`
- `freven_guest_handle_action` takes `ActionInput` and returns `ActionResult`
- `freven_guest_on_server_messages` takes `ServerMessageInput` and returns `ServerMessageResult`
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
- declared actions require `callbacks.action = true`
- `callbacks.action = true` requires at least one declared action
- max byte caps for negotiation/result/input payload before copying
- declared callback surface exactly matches the exported symbol surface
- dual-side lifecycle declarations are allowed; the runtime hosts the active side as a subset for the current session
- server-message routing uses the negotiated registration contract:
  inbound delivery only for declared server-readable channels, outbound sends only for declared server-writable channels and declared message ids

On decode/validation/contract errors, attach fails.
On lifecycle, action-call, or server-message contract faults, runtime disables that guest mod for the
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
