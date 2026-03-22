# freven_world_api

Stable world-stack-facing contracts for Freven builtin and compile-time world authoring.

`freven_world_api` carries the current world-facing declaration families that
live above the neutral SDK roots. It is intentionally world-owned, not a
neutral platform crate.

For runtime-loaded world mods, the canonical public contract lives in
`freven_world_guest` and the recommended authoring path is
`freven_world_guest_sdk` on Wasm.

Ownership boundaries:

- generic guest/runtime semantic roots live outside this crate
- volumetric topology/addressing truth lives in `freven_volumetric_sdk_types`
- standard block/profile vocabulary lives in `freven_block_sdk_types`
- `freven_world_api` consumes those lower-layer vocabularies for builtin /
  compile-time world authoring; it does not own them

Current state note:

- public standard block/profile vocabulary is owned by `freven_block_sdk_types`
- runtime-loaded block mutation/query/service contracts are owned by
  `freven_block_guest`
- builtin / compile-time block-facing traits and client interaction surfaces
  live in `freven_block_api`
- `freven_world_api` may still compose world-facing flows that reference those
  block-owned contracts, but it does not own block gameplay semantics

`freven_world_api` participates in the same world semantic system:

- deterministic registration via `ModContext`
- canonical capability declarations via `ModContext::declare_capability(...)`
- activation hooks via `on_start_client` / `on_start_server`
- runtime hooks via `on_tick_client` / `on_tick_server`
- dedicated client/server message phases
- world-facing registration and service surfaces that consume
  `BlockDescriptor` / `BlockRuntimeId` from `freven_block_sdk_types`
- action handlers over `ActionContext`, `BlockWorldView`, and `BlockAuthority`
- generic world runtime services plus world-facing composition over
  block-owned runtime service and mutation families used by runtime-loaded guests

Builtin / compile-time capability declarations use the same
`CapabilityDeclaration` model as `freven_world_guest`. When a builtin mod is
hosted from a resolved `mod.toml`, declared capability keys are validated
against that resolved capability table before the runtime records them.

Neutral boot/load/runtime truth still lives outside this crate. World
save/bootstrap metadata lives in `freven_world_sdk_types::save`, while
engine/app/bootstrap wiring does not belong here.

## Stability and semver stance

- Public runtime/mod contracts are treated as stable API.
- Additive changes are preferred to preserve downstream compatibility.
- Breaking changes require an intentional semver bump and release notes.
- While `< 1.0.0`, breaking changes may happen in minor releases but must be
  deliberate and documented.

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

## Documentation

* Repository docs: [https://github.com/frevenengine/freven-sdk/tree/main/docs](https://github.com/frevenengine/freven-sdk/tree/main/docs)
* Distribution / release policy: `docs/SDK_DISTRIBUTION.md`
* ABI docs: `docs/WASM_ABI_v1.md`, `docs/NATIVE_MOD_ABI_v1.md`, `docs/EXTERNAL_MOD_IPC_v1.md`
* Safety note: `docs/UNSAFE_NATIVE_MODS.md`
