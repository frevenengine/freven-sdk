# Freven SDK Docs

These docs describe the public Freven mod semantics and transport references.
Engine internals are private.

Read them in this order:

- [ARCHITECTURE.md](ARCHITECTURE.md): canonical Freven platform architecture
  and ownership vocabulary for engine, SDK, experiences, Vanilla, mods,
  content packs, script packs, standalone products, and save/world state
- [PACKAGE_BOUNDARIES.md](PACKAGE_BOUNDARIES.md): manifest, config schema,
  active config, content data, assets, generated cache, and save/world state
  ownership boundaries
- [VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md): stable visual asset
  keys, asset categories, dependency graph, validation, resolution, and
  renderer-backend boundaries
- [LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md): deterministic
  visual asset layering, override policy, conflict diagnostics, and
  server/client cosmetic rules
- [CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md): data-driven content
  add, replace, patch, append, disable, compatibility, and diagnostics
  semantics
- [WASM_AUTHORING.md](WASM_AUTHORING.md): recommended Wasm authoring paths for
  neutral guests and explicit world-stack guests
- [NEUTRAL_GUEST_CONTRACT_v1.md](NEUTRAL_GUEST_CONTRACT_v1.md): canonical
  neutral runtime-loaded guest semantics
- [GUEST_CONTRACT_v1.md](GUEST_CONTRACT_v1.md): canonical world-owned
  runtime-loaded guest semantics
- [MOD_CONFIG_v1.md](MOD_CONFIG_v1.md): public mod config schema,
  experience/stack authoring, resolution, and guest delivery semantics
- [WORLDGEN_PROVIDER_CONCURRENCY_v1.md](WORLDGEN_PROVIDER_CONCURRENCY_v1.md):
  canonical worldgen provider concurrency contract; current mode is
  `serial_session`
- [WORLD_ASYNC_SERVICE_MODEL_v1.md](WORLD_ASYNC_SERVICE_MODEL_v1.md):
  canonical guest-facing async/background computation model
- [SDK_DISTRIBUTION.md](SDK_DISTRIBUTION.md): distribution and release policy
- [WASM_ABI_v1.md](WASM_ABI_v1.md): world-stack Wasm transport reference
- [NATIVE_MOD_ABI_v1.md](NATIVE_MOD_ABI_v1.md): world-stack native transport reference
- [EXTERNAL_MOD_IPC_v1.md](EXTERNAL_MOD_IPC_v1.md): world-stack external companion-process reference
- [UNSAFE_NATIVE_MODS.md](UNSAFE_NATIVE_MODS.md): native trust / policy notes

`freven_mod_api` is the builtin / compile-time facade over the same semantic
system, including capability declarations through `ModContext::declare_capability`.
`freven_guest` remains the canonical neutral runtime-loaded guest contract.
World-shaped declarations now live under explicit `freven_world_*` ownership
rather than the neutral SDK roots. In that world stack, content registration,
world queries/mutations, terrain-write worldgen, and world runtime services
share one contract across builtin, Wasm, native, and external execution.
