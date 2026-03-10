# Freven SDK

This repository contains the **public Freven SDK** used to build experiences/mods.
The **engine source is private** and is **not** part of this repository.

## Public surface (GitHub-first)

Current SDK crates:
- `freven_api` - compile-time facade over the canonical public declaration model
- `freven_guest` - canonical transport-agnostic guest contract for runtime-loaded mods
- `freven_guest_sdk` - high-level guest authoring SDK for the normal Wasm mod path
- `freven_sdk_types` - pure shared SDK types
- `freven_std` - early stdlib helpers (**unstable**; depend only if you accept breakage)

Reference gameplay (vanilla experience) is shipped separately in the `freven-vanilla` repository.

## Depend on the SDK today (git tags)

Use tagged git dependencies until crates.io publishing begins:

```toml
[dependencies]
freven_api = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_api" }
freven_guest_sdk = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_guest_sdk" }
freven_sdk_types = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_sdk_types" }

# Low-level guest contract / transport work only:
# freven_guest = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_guest" }
# Optional / unstable:
# freven_std = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_std" }
```

See [docs/SDK_DISTRIBUTION.md](docs/SDK_DISTRIBUTION.md) for the rollout plan and release policy.

## Recommended authoring path

- Start with [docs/WASM_AUTHORING.md](docs/WASM_AUTHORING.md).
- Use `freven_guest_sdk` for normal mod authoring.
- Treat `freven_guest` and the transport ABI docs as reference material for low-level tests, fixtures, and runtime work.
- Treat `freven_api` and `freven_guest` as two facades over one declaration
  model by breadth; runtime guest execution still policy-gates provider-family
  hosting where services do not exist yet.
- Prefer Wasm for the primary safe path.
- Treat native and external transports as secondary integrations with narrower maturity/safety guarantees.

## Stability notes

- SDK is **pre-1.0**: breaking changes may happen in minor releases.
- Breaking changes will be called out in GitHub tag/release notes.
- Experimental areas will be labeled explicitly in docs/comments.
