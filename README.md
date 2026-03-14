# Freven SDK

This repository contains the public Freven SDK for mod and experience authoring.
The engine source is private and is not part of this repository.

## Public crates

- `freven_guest_sdk`: high-level guest SDK for the recommended public Wasm path
- `freven_guest`: canonical transport-agnostic guest contract for runtime-loaded mods
- `freven_mod_api`: builtin / compile-time facade over the same semantic model
- `freven_sdk_types`: neutral shared SDK types and observability helpers
- `freven_world_guest_sdk`: explicit world-owned guest authoring surface
- `freven_world_guest`: explicit world-owned runtime-loaded world contract
- `freven_world_api`: explicit world-owned builtin / compile-time facade
- `freven_world_sdk_types`: explicit world-owned block/voxel shared types

`freven_api` has been retired. The public crate name is now `freven_mod_api`
so the builtin / compile-time surface is not mistaken for the whole SDK.

Reference gameplay lives in the separate `freven-vanilla` repository. Stage 01
of the platform boundary reset removed block, voxel, controller, world-query,
world-command, and first-party gameplay helpers from the neutral SDK roots.
Vanilla-owned break/place payload helpers, humanoid input codecs, and first-party
ids now live in `freven-vanilla`, not in the SDK repository.

## Recommended authoring path

Start with [docs/WASM_AUTHORING.md](docs/WASM_AUTHORING.md).

- Use `freven_guest_sdk` for normal mod authoring.
- Treat Wasm as the polished safe public path.
- Treat `freven_guest` plus the ABI / IPC docs as low-level reference material
  for fixtures, runtime work, and transport adapters.
- Use `freven_mod_api` when you are authoring builtin / compile-time mods or
  host-side semantic integrations.
- Use the `freven_world_*` crates only when you are intentionally targeting the
  current world stack; they are explicit world-owned surfaces, not neutral SDK roots.
- Treat the `freven_world_*` crates as thin world overlays over the neutral SDK roots,
  not as replacement top-level SDK monoliths.
- Treat native and external execution as secondary trust / execution paths, not
  as equal onboarding stories.
- Treat builtin mods as the same semantic system through a different execution
  path, not as a separate mod model.
- Neutral roots now cover generic lifecycle, components, messages, channels,
  capabilities, session identity, and observability. World-shaped declarations
  live behind explicit `freven_world_*` ownership.

## Depend on the SDK today

Use tagged git dependencies until crates.io publishing begins:

```toml
[dependencies]
freven_guest_sdk = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_guest_sdk" }
freven_mod_api = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_mod_api" }
freven_sdk_types = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_sdk_types" }

# Low-level guest contract work only:
# freven_guest = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_guest" }

# Current world-stack integrations only:
# freven_world_guest_sdk = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_world_guest_sdk" }
# freven_world_api = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_world_api" }
```

See [docs/SDK_DISTRIBUTION.md](docs/SDK_DISTRIBUTION.md) for release policy.

## Stability notes

- SDK is pre-1.0.
- Breaking changes will be called out in release notes.
- Experimental areas are labeled explicitly in docs and code.
