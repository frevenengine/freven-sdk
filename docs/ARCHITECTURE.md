# Freven Architecture Model

This document defines the recommended public architecture vocabulary for Freven
SDK authors.

Freven is a platform for experiences, not only a single game or a thin mod
loader. The most important boundary is that the engine provides generic
capabilities, the SDK defines public contracts, and gameplay/content layers own
their authored meaning.

This document intentionally stays at the architecture and ownership level.
Detailed file ownership, visual asset schemas, override rules, and patch/merge
semantics are defined by follow-up documents.

## Goals

- Keep engine/runtime implementation details out of normal author APIs.
- Keep Vanilla gameplay, content, visuals, and style out of engine core.
- Give mods, content packs, standalone games, and total conversions one shared
  vocabulary.
- Make data, assets, config, and save state boundaries explicit before rc10
  visual/data work grows.
- Preserve a future path for creator-friendly scripting without making it a
  separate platform model.

## Layer model

| Layer | Owns | Does not own |
| --- | --- | --- |
| Engine / platform | runtime host, renderer, networking, storage, asset resolver, scheduling, sandboxing, diagnostics, cache, authoritative apply paths | Vanilla gameplay, first-party content, author-facing schemas, mod policy decisions baked as gameplay meaning |
| SDK | public contracts, schemas, namespaced identities, guest APIs, capability vocabulary, author-facing validation model | engine internals, renderer slots, Bevy/wgpu types, concrete Vanilla content |
| Game / gameplay SDK roots | explicit world, volumetric, block, avatar, gameplay-state, item, UI, input, effect, and save contracts above the neutral SDK roots | neutral platform ownership, renderer implementation, first-party Vanilla semantics |
| Experience | a concrete playable root such as Vanilla, a minigame, a total conversion, or a standalone game experience | engine implementation details |
| Vanilla | first-party reference experience, default gameplay/content/style, reference integration patterns | platform ownership, required dependency for all games |
| Standalone product | distribution boundary, bundled engine/devkit/runtime pieces, selected experience(s), default config, packaged content/assets | a requirement to include Vanilla |
| Mod | code/data/assets package that extends or patches a target experience through declared dependencies and capabilities | unrestricted engine access, implicit ownership of another namespace |
| Content pack | data/assets-only package for adding or overriding authored content through explicit rules | executable code, runtime authority by itself |
| Script pack | future creator-friendly scripting package that uses the same public contracts and capability model | a separate hidden runtime model |
| Save/world state | persistent runtime state bound to a world/session and compatible content identity | shipped content defaults, manifest metadata, generated asset cache |

## Core definitions

### Engine / platform

The engine is the generic host implementation. It can know how to render, load,
resolve, validate, cache, simulate, network, store, sandbox, and apply runtime
commands.

The engine must not need to know what `grass`, `stone`, `vanilla:pickaxe`, or a
Vanilla humanoid means. It should operate on resolved contracts, validated data,
namespaced identities, internal handles, and authoritative runtime state.

Normal author APIs must not expose renderer-internal slots, palette ids, atlas
coordinates, Bevy-specific component names, or other host implementation details.

### SDK

The SDK is the public contract layer. It defines what authors can depend on:
guest contracts, SDK crates, schemas, namespaced keys, config schema vocabulary,
runtime service requests, capability declarations, diagnostics vocabulary, and
data model conventions.

The SDK can describe public meaning. It should not implement the engine runtime,
own first-party Vanilla content, or force every experience to inherit Vanilla.

### Game / gameplay SDK roots

Neutral SDK roots describe platform-shaped concepts such as lifecycle, messages,
channels, capabilities, session identity, and observability.

Game SDK roots describe explicit gameplay-owned concepts above the neutral
platform layer. Today that includes world, volumetric, block, avatar, and
gameplay-state contracts. Future roots may own items, entities, UI,
effects, input, save contracts, or other game-facing APIs.

World/gameplay-specific concepts live under explicit owner roots such as
`freven_world_*`, `freven_volumetric_*`, `freven_block_*`, and future equivalent
roots. This keeps block/voxel/world assumptions from becoming the neutral Freven
platform story.

### Experience

An experience is a concrete playable root. Examples include:

- first-party Vanilla;
- a Vanilla-derived modded experience;
- a minigame;
- a total conversion;
- a standalone game with no Vanilla dependency.

An experience owns its active mod/content stack, default authored config, content
baseline, and gameplay identity. It can depend on other packages, but it should
do so explicitly.

### Vanilla

Vanilla is the first-party reference experience. It is important because it shows
how the platform is meant to be used, but it is not the platform itself.

Vanilla may own blocks, items, recipes, movement style, visuals, UI decisions,
and reference content. Engine and neutral SDK layers must not hardcode those
choices. Standalone games and total conversions must be able to replace Vanilla
completely.

### Mod

A mod is an extension unit. It may include executable guest code, content data,
assets, config schema, and migrations depending on its declared artifact,
execution, trust, surfaces, and capabilities.

A mod extends or patches a target experience through explicit dependencies,
declared capabilities, namespaced identities, and resolver rules. A mod should
not silently take ownership of another namespace or rely on load-order accidents.

### Content pack

A content pack is data/assets only. It can add, replace, or patch authored
content only through explicit content and asset rules.

A content pack is the right shape for simple visual/content changes that do not
need executable code. It must not get runtime authority just because it can
provide data or assets.

### Script pack

A script pack is a future creator-friendly executable layer for languages above
the lower-level Rust/Wasm path.

Script packs should use the same platform contracts, capability checks,
namespaces, budgets, diagnostics, and authority model as other guest execution
paths. They should not become a second hidden mod architecture.

### Standalone product

A standalone product is a packaging and distribution boundary. It can bundle a
runtime, selected experiences, default config, content, assets, and product shell
files.

A standalone product may include Vanilla, extend Vanilla, or ship a zero-Vanilla
experience. Product packaging must not imply that Vanilla is required by the
engine or SDK.

## Package and state boundaries

The architecture uses these high-level boundaries:

| Boundary | Purpose |
| --- | --- |
| Manifest | package identity, version, dependencies, artifact/execution/trust metadata, surfaces, entrypoint, capability requests, schema references |
| Config schema | supported settings, defaults, validation constraints, reload policy, authority |
| Active config | selected values resolved from the active experience or stack |
| Content data | authored gameplay/visual definitions such as blocks, items, recipes, materials, models, providers, families |
| Assets | referenced files such as images, models, shaders, sounds, fonts, and other resource data |
| Generated cache | host/devkit-owned derived data such as atlases, load plans, fingerprints, and compiled cache artifacts |
| Save/world state | persistent runtime state produced by play and simulation |

Detailed rules for these file and state boundaries are defined in
[PACKAGE_BOUNDARIES.md](PACKAGE_BOUNDARIES.md). The shared visual asset identity
and dependency model is defined in
[VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md). The core architecture rule is
that shipped defaults, active config, generated cache, and save/world state are
not the same thing.

## Composition model

A typical Freven launch resolves in this conceptual order:

1. Select a product or install boundary.
2. Select an experience or experience stack.
3. Resolve declared packages, dependencies, trust policy, and active sides.
4. Resolve config schemas and active config values.
5. Resolve content data and visual assets through deterministic layer rules.
6. Build host-internal load plans, caches, and runtime handles.
7. Start the server/client runtime with only the resolved public contracts.
8. Persist runtime state into save/world state, not back into shipped content.

This model should work for Vanilla, Vanilla mods, content packs, total
conversions, and standalone games.

## Authority and compatibility

Author-facing identity should be namespaced and stable. Runtime and tooling must
distinguish at least these categories:

- authoritative server-required content and gameplay contracts;
- client-local cosmetic assets where allowed;
- generated cache that can be rebuilt;
- save/world state that may require migration when authoritative content changes.

A client-local texture override should not be treated the same as a server-side
block definition change. A generated atlas should not be treated as authored
source. A save migration should not be expressed by editing a shipped content
file at runtime.

## Non-goals for this document

This document does not define the full schemas for:

- manifest vs config vs content data vs assets vs save state;
- detailed material/model/texture/shader schemas beyond the shared visual asset
  model;
- layered asset override implementation details beyond the v1 policy model;
- data-driven content patch/merge implementation details beyond the v1 semantic model;
- beginner-friendly data authoring formats;
- future scripting-language bindings.

Those are separate SDK and DevKit documents/issues that should reference this
architecture vocabulary instead of redefining it.

## Practical rules for contributors

- Put generic host capabilities in the engine, not in Vanilla.
- Put stable author-facing contracts in the SDK, not in engine internals.
- Put first-party gameplay/content/style in Vanilla, not in the engine.
- Put package identity and capability requests in manifests, not active runtime
  config.
- Put active values in experience/stack config, not `mod.toml`.
- Put authored content in content data and assets, not save/world state.
- Put generated resolver output in cache/load-plan data, not authored source.
- Keep every override, patch, and dependency decision explicit and diagnosable.
- Prefer namespaced keys over renderer slots or raw host ids in public APIs.
- Design every new rc10 visual/data feature so a zero-Vanilla standalone game can
  use it.
