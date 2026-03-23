# Freven SDK

`freven-sdk` is the public SDK and contract surface for Freven mod and experience authoring.

Freven is being built as a **platform for experiences**:
- a neutral platform layer
- optional world/game stacks layered on top
- concrete first-party or third-party experiences
- future mod/module/service composition above those layers

This repository contains the public author-facing SDK for that direction.
Freven engine internals remain private and are **not** part of this repository.

## Repository role

`freven-sdk` is the public contract layer of Freven.

It is responsible for:
- guest-facing APIs
- public authoring surfaces
- shared SDK contracts and types
- public Wasm mod authoring
- builtin / compile-time integration surfaces
- explicit world-owned SDK overlays above the neutral SDK roots

It is **not**:
- the engine implementation
- the first-party Vanilla gameplay repository
- the full Freven runtime host
- the full Freven world/game stack implementation

## Freven architecture at a glance

The current long-term direction is:

- **engine/platform layer**: neutral runtime/platform substrate
- **world stacks**: explicit world/game-specific layers above the platform
- **experiences**: concrete games or modes built on top
- **mods/modules/services**: extension units that can evolve into a broader ecosystem

Within that model:

- `freven-sdk` provides the **public SDK surface**
- `freven-vanilla` provides the **first-party reference experience**
- Freven engine internals remain private

## Public SDK surfaces

The repository currently exposes two kinds of public SDK surface:

### Neutral SDK roots

These cover generic platform-shaped authoring concerns.

- `freven_guest_sdk` — high-level neutral guest SDK for the public Wasm path
- `freven_guest` — neutral transport-agnostic guest contract for runtime-loaded mods
- `freven_mod_api` — builtin / compile-time facade over the same neutral semantic model
- `freven_sdk_types` — neutral shared SDK types and observability helpers

### Explicit world / volumetric / block-owned SDK roots

These cover the current non-neutral gameplay/runtime stacks.

#### Generic world-owned roots

- `freven_world_guest_sdk` — explicit world-owned guest authoring surface
- `freven_world_guest` — explicit world-owned runtime-loaded world contract
- `freven_world_api` — explicit world-owned builtin / compile-time facade
- `freven_world_sdk_types` — explicit generic world/shared types

#### Volumetric-owned roots

- `freven_volumetric_sdk_types` — explicit volumetric topology/addressing types

#### Reusable block-owned roots

- `freven_block_sdk_types` — public standard block gameplay vocabulary
- `freven_block_guest` — runtime-loaded block mutation/query/service contracts
- `freven_block_api` — builtin / compile-time block-facing traits and client
  interaction surfaces

## How to think about the split

The important rule is:

- neutral SDK roots describe **platform-shaped** concepts
- `freven_world_*` roots describe **explicit world-owned** concepts

That means world-, volumetric-, and block-specific authoring is no longer
presented as the neutral top-level SDK story, and those ownership lines are now
explicitly separated instead of being treated as one mixed world/block surface.

Reference first-party gameplay lives in the separate `freven-vanilla` repository.
Vanilla-owned break/place payload helpers, humanoid input codecs, and first-party ids
do **not** live in the neutral SDK roots.

## Which path should authors use?

Start with [docs/WASM_AUTHORING.md](docs/WASM_AUTHORING.md).

Recommended path:

- use **`freven_guest_sdk`** for neutral runtime-loaded Wasm guests
- use **`freven_world_guest_sdk`** for current gameplay/world-stack mods
- treat **Wasm** as the polished safe public path
- treat **native/external execution** as secondary trust / execution paths, not as equal onboarding stories
- treat **builtin / compile-time mods** as the same semantic model through a different execution path, not as a separate mod model

## Which crate should I pick?

Use:

- **`freven_guest_sdk`** if you need lifecycle, messages, components, channels, capabilities, session identity, and observability
- **`freven_world_guest_sdk`** if you are writing a runtime-loaded gameplay/world-stack mod against the current world surface
- **`freven_mod_api`** for builtin / compile-time neutral integrations
- **`freven_world_api`** for builtin / compile-time gameplay/world authoring
- **`freven_guest`** only when you need the low-level neutral contract directly
- **`freven_world_guest`** only when you need the low-level explicit world contract directly

For most public runtime-loaded mod authors, the intended starting point is:
- `freven_guest_sdk`, or
- `freven_world_guest_sdk`

## Depend on the SDK today

Use tagged git dependencies until crates.io publishing begins:

```toml
[dependencies]
freven_guest_sdk = { git = "https://github.com/frevenengine/freven-sdk.git", tag = "v0.1.2-rc3", package = "freven_guest_sdk" }
freven_mod_api   = { git = "https://github.com/frevenengine/freven-sdk.git", tag = "v0.1.2-rc3", package = "freven_mod_api" }
freven_sdk_types = { git = "https://github.com/frevenengine/freven-sdk.git", tag = "v0.1.2-rc3", package = "freven_sdk_types" }

# Low-level guest contract work only:
# freven_guest = { git = "https://github.com/frevenengine/freven-sdk.git", tag = "v0.1.2-rc3", package = "freven_guest" }

# Current world-stack integrations only:
# freven_world_guest_sdk = { git = "https://github.com/frevenengine/freven-sdk.git", tag = "v0.1.2-rc3", package = "freven_world_guest_sdk" }
# freven_world_api       = { git = "https://github.com/frevenengine/freven-sdk.git", tag = "v0.1.2-rc3", package = "freven_world_api" }
````

See [docs/SDK_DISTRIBUTION.md](docs/SDK_DISTRIBUTION.md) for release policy.

## Current status

* the SDK is pre-1.0
* breaking changes will be called out in release notes
* experimental areas are labeled explicitly in docs and code
* the public Wasm path is the recommended public authoring path today
* engine internals remain private
* other Freven repositories may use different licenses

## Related repositories

* `freven-sdk` — public SDK and contract surface
* `freven-vanilla` — first-party reference experience
* Freven engine repositories — private implementation/runtime side

## License

This repository is licensed under the Apache License, Version 2.0.
See [LICENSE](LICENSE) for the full text.
