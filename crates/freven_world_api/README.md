# freven_world_api

Explicit world-owned contracts for Freven builtin and compile-time world authoring.

`freven_world_api` carries the current world-stack-facing declaration families
that live above the neutral SDK roots. It is intentionally world-owned, not a
neutral platform crate.

For runtime-loaded world mods, the canonical public contract lives in
`freven_world_guest` and the recommended authoring path is
`freven_world_guest_sdk` on Wasm.

`freven_world_api` participates in the same world semantic system:

- deterministic registration via `ModContext`
- canonical capability declarations via `ModContext::declare_capability(...)`
- activation hooks via `on_start_client` / `on_start_server`
- runtime hooks via `on_tick_client` / `on_tick_server`
- dedicated client/server message phases
- block/content registration via `BlockDescriptor` + `BlockRuntimeId`
- action handlers over `ActionContext`, `WorldView`, and `WorldAuthority`
- world runtime services and world mutation output families shared with
  runtime-loaded guests

Builtin / compile-time capability declarations use the same
`CapabilityDeclaration` model as `freven_world_guest`. When a builtin mod is hosted
from a resolved `mod.toml`, declared capability keys are validated against that
resolved capability table before the runtime records them.

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
use freven_mod_api::{ModSide, Side};
use freven_world_api::{BlockRuntimeId, WorldMutation};

let mutation = WorldMutation::SetBlock {
    pos: (4, 80, 4),
    block_id: BlockRuntimeId(7),
    expected_old: None,
};
assert!(matches!(mutation, WorldMutation::SetBlock { .. }));
assert!(ModSide::Both.matches(Side::Client));
```

## Documentation

- Repository docs: <https://github.com/frevenengine/freven-sdk/tree/main/docs>
- Distribution / release policy: `docs/SDK_DISTRIBUTION.md`
- ABI docs: `docs/WASM_ABI_v1.md`, `docs/NATIVE_MOD_ABI_v1.md`, `docs/EXTERNAL_MOD_IPC_v1.md`
- Safety note: `docs/UNSAFE_NATIVE_MODS.md`
