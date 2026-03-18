# Freven SDK Docs

These docs describe the public Freven mod semantics and transport references.
Engine internals are private.

Read them in this order:

- [WASM_AUTHORING.md](WASM_AUTHORING.md): recommended Wasm authoring paths for
  neutral guests and explicit world-stack guests
- [NEUTRAL_GUEST_CONTRACT_v1.md](NEUTRAL_GUEST_CONTRACT_v1.md): canonical
  neutral runtime-loaded guest semantics
- [GUEST_CONTRACT_v1.md](GUEST_CONTRACT_v1.md): canonical world-owned
  runtime-loaded guest semantics
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
