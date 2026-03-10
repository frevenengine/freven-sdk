# WASM ABI v1

This document defines the Freven WASM mod ABI for WP7A.

The canonical public guest contract is `freven_guest` and is documented in
`GUEST_CONTRACT_v1.md`. This document covers the Wasm transport mapping for that
contract.

For normal mod authoring, use `freven_guest_sdk` and the guide in
`WASM_AUTHORING.md`. This document is transport reference material, not the
recommended getting-started path.

## Scope

- Supports negotiation, declaration registration, lifecycle callbacks, side-specific message callbacks, and action handling over Wasm
  ptr/len calls.
- Host runs modules with no WASI and no host imports by default.
- `[capabilities]` in `mod.toml` is enforced by runtime with a strict allowlist.

## Required exports

A module must export these symbols:

- `freven_guest_alloc(size: u32) -> u32`
- `freven_guest_dealloc(ptr: u32, size: u32)`
- `freven_guest_negotiate(ptr: u32, len: u32) -> u64`
- `freven_guest_handle_action(ptr: u32, len: u32) -> u64` if `callbacks.action = true`
- `freven_guest_on_client_messages(ptr: u32, len: u32) -> u64` if `callbacks.messages.client = true`
- `freven_guest_on_server_messages(ptr: u32, len: u32) -> u64` if `callbacks.messages.server = true`
- linear memory export named `memory`

The negotiated `GuestDescription` must also be internally coherent:

- declared actions require `callbacks.action = true`
- `callbacks.action = true` requires at least one declared action

Optional lifecycle exports:

- `freven_guest_on_start_client(ptr: u32, len: u32) -> u64`
- `freven_guest_on_start_server(ptr: u32, len: u32) -> u64`
- `freven_guest_on_tick_client(ptr: u32, len: u32) -> u64`
- `freven_guest_on_tick_server(ptr: u32, len: u32) -> u64`

`freven_guest_negotiate`, lifecycle callbacks, and `freven_guest_handle_action`
return packed `(ptr, len)` as:

- `((ptr as u64) << 32) | (len as u64)`

The host copies returned bytes from guest memory and then calls
`freven_guest_dealloc(ptr, len)`.

## Encoding

ABI payloads are `postcard` encoded values from `freven_guest`.

### Negotiation (`freven_guest_negotiate`)

Input: `NegotiationRequest`

Output: `NegotiationResponse`

Host behavior:

- validates `selected_contract_version`
- validates `GuestDescription.callbacks` against exported Wasm symbols
- registers `GuestDescription.registration` into the canonical host runtime
- maps runtime action kind to `registration.actions[].binding_id` for callback dispatch

### Lifecycle inputs and outputs

- `freven_guest_on_start_*` input: `StartInput`
- `freven_guest_on_tick_*` input: `TickInput`
- lifecycle output: `LifecycleAck`

Lifecycle is intentionally ack-only in guest contract v1. Returning any richer
lifecycle effect payload is not part of the contract.

### Action input (`freven_guest_handle_action` input bytes)

`ActionInput`:

- `binding_id: u32`
- `player_id: u64`
- `level_id: u32`
- `stream_epoch: u32`
- `action_seq: u32`
- `at_input_seq: u32`
- `player_position_m: Option<[f32; 3]>`
- `payload: &[u8]` (opaque client/server action payload)

### Message callbacks

- `freven_guest_on_client_messages` input: `ClientMessageInput`, output: `ClientMessageResult`
- `freven_guest_on_server_messages` input: `ServerMessageInput`, output: `ServerMessageResult`
- the host routes inbound mod messages only for the guest's declared side-appropriate readable channels
- guest outbound sends must use declared message ids and declared side-appropriate writable channels
- unsupported message-scope mapping is rejected explicitly; the runtime does not silently coerce scope

### Action result (`freven_guest_handle_action` return bytes)

`ActionResult`:

- `outcome: ActionOutcome` (`applied` or `rejected`)
- `effects: EffectBatch`

`WorldEffect` is a `postcard`-encoded Rust enum carried inside
`EffectBatch.world`.

ABI rule: enum variant order is ABI-significant.
- Do NOT reorder variants.
- Do NOT rename variants expecting any effect on binary encoding.
- Only append new variants at the end.

Currently supported variants:

- `SetBlock { pos: (i32, i32, i32), block_id: u8 }`

Host applies `SetBlock` effects through server world-edit APIs. Any
decode/trap/validation/apply failure disables that guest for the runtime
session and the action rejects.

## Capability policy (implemented in `freven_runtime_wasm`)

Runtime accepts only these capability keys:

- `max_call_millis` (integer, must be `> 0`, cannot exceed host policy max)
- `max_linear_memory_bytes` (integer, must be `> 0`, cannot exceed host policy max)
- `allow_unstable` (boolean, must be `false`)

Unknown keys are rejected. Invalid types are rejected.
Declared capability keys must also exist in the resolved capability table; the
runtime reports that as an explicit capability-declaration error rather than a duplicate-key error.

Current host policy maxima/defaults:

- `max_call_millis`: `25`
- `max_linear_memory_bytes`: `4 MiB`
- `max_negotiation_bytes`: `64 KiB`
- `max_result_bytes`: `256 KiB`
- `max_input_payload_bytes`: `64 KiB`
- `max_world_effects` per action result: `128`

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
- maximum output bytes for negotiation, lifecycle, and action result payloads

Guest modules must return packed `(ptr, len)` ranges that are valid and within
host-configured size limits. If a call exceeds limits or violates the contract,
the runtime may disable that guest for the current runtime session.

Time budgets may be enforced using Wasmtime epoch deadlines driven by host epoch ticking
(implementation detail only; ABI contract is unchanged).
