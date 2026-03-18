# freven_guest_sdk

Neutral guest authoring helpers for Freven runtime-loaded mods.

Use this crate for the normal Wasm authoring path. It sits on top of the
canonical `freven_guest` contract and hides the transport boilerplate:

- canonical declaration types for components, messages, channels, and
  capabilities
- lifecycle and message contract types
- session identity and observability helpers
- logging macros routed through the host observability bridge

This crate is intentionally neutral after the platform/world boundary reset.
World-shaped declarations do not live here.

Reach for `freven_guest` directly only when you are implementing or testing the
raw neutral guest contract itself.

Use `freven_world_guest_sdk` when you need:

- blocks or voxel content registration
- action handlers that issue world mutations
- worldgen
- character controllers
- client-control providers
- world/runtime services and world queries

## Minimal usage

```rust
freven_guest_sdk::log_info!("hello from a neutral guest");
```

## Current boundaries

- neutral declarations cover components, messages, channels, and capability keys
- lifecycle and message hooks remain transport-agnostic contract truth
- logging and observability stay available without importing world semantics
- world-shaped declarations live behind the explicit `freven_world_guest_sdk`
  layer instead of the neutral SDK root
