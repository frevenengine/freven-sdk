# freven_api

Stable SDK contracts for Freven experiences and compile-time mods.

## Publishing status

`freven_api` is intentionally `publish = false` right now. The crate can be made
publishable after workspace package metadata and release ownership policy are
explicitly configured for public distribution.

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
