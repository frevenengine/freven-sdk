# freven_mod_api

Stable SDK contracts for Freven builtin and compile-time mod authoring.

`freven_mod_api` defines the engine-agnostic public contracts used by Freven
experiences, builtin mods, and compile-time registrations. It is the semantic
facade for the in-process authoring path, not the name of the whole SDK.

For runtime-loaded mods, the canonical public contract lives in `freven_guest`
and the recommended public authoring path is `freven_guest_sdk` on Wasm.

`freven_mod_api` still participates in the same semantic system:

- deterministic registration via `ModContext`
- activation hooks via `on_start_client` / `on_start_server`
- runtime hooks via `on_tick_client` / `on_tick_server`
- dedicated client/server message phases
- the same declaration families, runtime output model, and observability model
  used by runtime-loaded guests

`ExperienceSpec` in this crate is a compile-time convenience surface. Canonical
boot/load/runtime truth lives in the engine runtime activation model, not here.

Engine/app/bootstrap wiring does not belong in this crate.

## Stability and semver stance

- Public runtime/mod contracts are treated as stable API.
- Additive changes are preferred to preserve downstream compatibility.
- Breaking changes require an intentional semver bump and release notes.
- While `< 1.0.0`, breaking changes may happen in minor releases but must be
  deliberate and documented.

## Minimal usage

```rust
use freven_mod_api::{ActionKindId, ModSide, Side};

let kind = ActionKindId(7);
assert_eq!(kind.raw(), 7);
assert!(ModSide::Both.matches(Side::Client));
```

## Documentation

- Repository docs: <https://github.com/frevenengine/freven-sdk/tree/main/docs>
- Distribution / release policy: `docs/SDK_DISTRIBUTION.md`
- ABI docs: `docs/WASM_ABI_v1.md`, `docs/NATIVE_MOD_ABI_v1.md`, `docs/EXTERNAL_MOD_IPC_v1.md`
- Safety note: `docs/UNSAFE_NATIVE_MODS.md`
