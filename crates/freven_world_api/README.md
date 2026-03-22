# freven_world_api

Stable world-facing contracts for Freven builtin and compile-time authoring.

`freven_world_api` is an explicit world-owned crate. It is not a neutral
platform root and it is not the owner of lower-layer volumetric or block
vocabulary.

For runtime-loaded world mods, the canonical public contract lives in
`freven_world_guest`, and the recommended Wasm authoring path is
`freven_world_guest_sdk`.

## Ownership boundaries

Lower-layer ownership is explicit:

- generic world/save/session truth lives in `freven_world_sdk_types`
- volumetric topology/addressing truth lives in `freven_volumetric_sdk_types`
- public standard block gameplay vocabulary lives in `freven_block_sdk_types`
- runtime-loaded block mutation/query/service contracts live in
  `freven_block_guest`
- builtin / compile-time block-facing traits and client interaction surfaces
  live in `freven_block_api`
- engine-side block registry/runtime-id/apply/policy truth lives in
  `freven_block_runtime`

`freven_world_api` composes over those lower-layer surfaces for builtin /
compile-time world authoring. It does not own them.

## What `freven_world_api` owns

`freven_world_api` owns the world-facing builtin / compile-time facade for:

- deterministic registration via `ModContext`
- canonical capability declarations
- lifecycle hooks (`on_start_client`, `on_start_server`, `on_tick_client`,
  `on_tick_server`)
- world-facing provider registration such as worldgen, character controller, and
  client-control provider families
- action-handler contracts over `ActionContext`
- generic world-facing runtime-service composition for builtin / compile-time
  integrations

## Block-family composition note

Some world-facing flows still compose over block-owned families.

Examples:

- `ActionContext::block_id_by_key(...)`
- `WorldServiceRequest::Block(...)`
- `WorldServiceResponse::Block(...)`
- `apply_block_mutations(...)`

These are composition points over block-owned families. They do not make
`freven_world_api` the owner of standard block gameplay semantics.

## Current state

Stage 4.5 ownership is:

- `freven_world_sdk_types` = generic world truth
- `freven_volumetric_sdk_types` = volumetric topology truth
- `freven_block_sdk_types` = public standard block vocabulary
- `freven_block_guest` = runtime-loaded block query/service/mutation contracts
- `freven_block_api` = builtin / compile-time block-facing traits and client
  block interaction surfaces
- `freven_block_runtime` = engine-owned block runtime truth
- `freven_world_api` = world-facing builtin / compile-time composition layer

## Minimal usage

```rust
use freven_block_guest::BlockMutation;
use freven_block_sdk_types::BlockRuntimeId;
use freven_mod_api::{ModSide, Side};

let mutation = BlockMutation::SetBlock {
    pos: (4, 80, 4),
    block_id: BlockRuntimeId(7),
    expected_old: None,
};

assert!(matches!(mutation, BlockMutation::SetBlock { .. }));
assert!(ModSide::Both.matches(Side::Client));
```

## Stability and semver stance

* Public runtime/mod contracts are treated as stable API.
* Additive changes are preferred to preserve downstream compatibility.
* Breaking changes require an intentional semver bump and release notes.
* While `< 1.0.0`, breaking changes may happen in minor releases but must still
  be deliberate and documented.

## Documentation

* Repository docs: `docs/`
* Distribution / release policy: `docs/SDK_DISTRIBUTION.md`
* ABI docs: `docs/WASM_ABI_v1.md`, `docs/NATIVE_MOD_ABI_v1.md`,
  `docs/EXTERNAL_MOD_IPC_v1.md`
* Safety note: `docs/UNSAFE_NATIVE_MODS.md`
