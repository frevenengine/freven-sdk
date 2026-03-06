# freven_api

Stable SDK contracts for Freven experiences and compile-time mods.

`freven_api` defines the engine-agnostic public contracts used by Freven
experiences and compile-time mods. It is intended to stay stable so mods can be
built and versioned independently from engine internals.

The lifecycle contract is intentionally small:
- registration via `ModContext`
- activation via `on_start_common`, `on_start_client`, `on_start_server`
- runtime via `on_tick_client`, `on_tick_server`, and explicit action dispatch

Engine/app/bootstrap wiring does not belong in this crate.

For runtime-loaded guests, the canonical public contract lives in
`freven_guest`, not in transport-specific ABI docs.

## Stability and semver stance

- Public runtime/mod contracts are treated as stable API.
- Additive changes are preferred to preserve downstream compatibility.
- Breaking changes require an intentional semver bump and release notes.
- While `< 1.0.0`, breaking changes may happen in minor releases but must be
  deliberate and documented.

## Minimal usage

```rust
use freven_api::{ActionKindId, ModSide, Side};

let kind = ActionKindId(7);
assert_eq!(kind.raw(), 7);
assert!(ModSide::Both.matches(Side::Client));
```

## Documentation

- Repository docs: <https://github.com/frevenengine/freven-sdk/tree/main/docs>
- Distribution / release policy: `docs/SDK_DISTRIBUTION.md`
- ABI docs: `docs/WASM_ABI_v1.md`, `docs/NATIVE_MOD_ABI_v1.md`, `docs/EXTERNAL_MOD_IPC_v1.md`
- Safety note: `docs/UNSAFE_NATIVE_MODS.md`

## Public contract expectations

- APIs in this crate define host/runtime contracts and must stay engine-agnostic.
- New capabilities should be additive when possible.
- Any breaking change requires an explicit version bump and changelog note.

## API evolution policy

### Stable vs experimental

- Stable: items documented and used as runtime-facing contracts.
- Experimental: items gated behind `experimental_*` naming or explicit docs note.
  Experimental items may change faster between pre-1.0 releases.

### `#[non_exhaustive]` strategy

- Public enums and structs expected to grow should use `#[non_exhaustive]` to
  preserve forward compatibility for downstream users.
- When `#[non_exhaustive]` is not used, new variants/fields are treated as
  breaking changes.

### Pre-1.0 breaking expectations

- While version is `< 1.0.0`, semver still applies.
- Breaking API changes are allowed in minor releases, but must be intentional,
  documented, and minimized.
- Patch releases must remain backward compatible.
