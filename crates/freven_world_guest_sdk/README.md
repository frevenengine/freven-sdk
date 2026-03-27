# freven_world_guest_sdk

High-level guest authoring helpers for Freven runtime-loaded world mods.

Use this crate for the normal Wasm authoring path. It sits on top of the
canonical `freven_world_guest` contract and hides the transport boilerplate:

- guest alloc/dealloc exports
- `postcard` encode/decode plumbing
- Wasm export table wiring
- native in-process export wiring for low-level fixtures/tests
- canonical declaration builders for blocks/components/messages/worldgen/
  character-controllers/client-control-providers/channels/actions/capabilities
- lifecycle/action/message dispatch lookup
- export-surface validation against the canonical `GuestDescription`
- helpers for world queries, world mutations, and terrain-write worldgen

This crate is intentionally world-owned. Neutral guest authoring stops at
generic lifecycle/messages/components/channels/capabilities/observability; the
world-specific declaration families and helpers live here instead.

## Minimal example

```rust
use freven_world_guest_sdk::{ActionContext, ActionResponse};

fn handle_action(ctx: ActionContext<'_>) -> ActionResponse {
    let _ = ctx.player_id();
    ActionResponse::applied().set_block((4, 80, 4), 1)
}

freven_world_guest_sdk::wasm_guest!(
    guest_id: "freven.example.wasm",
    lifecycle: {
        start_server: |_| {},
        tick_server: |tick| {
            let _ = tick.tick;
        },
    },
    actions: {
        "freven.example:set_block" => {
            binding_id: 1,
            handler: handle_action,
        },
    },
);
```

`wasm_guest!` is the normal public authoring path: the guest id, registration
families, callback families, negotiated `GuestDescription`, and emitted Wasm
export surface all come from that one declaration.

Provider families are authored on that same path through `registration`:

```rust
registration: {
    worldgen: "freven.example:flat" => generate_worldgen,
    character_controller: "freven.example:walker" => {
        init: init_character_controller,
        step: step_character_controller,
    },
    client_control_provider: "freven.example:controls" => sample_client_control,
}
```

`GuestModule` plus `export_wasm_guest!(...)` / `export_native_guest!(...)`
remain available for lower-level fixtures and ABI-focused tests when you
intentionally need to wire the raw surface yourself.

## Current boundaries

- Lifecycle hooks return `LifecycleResult`.
- `registration.actions` and `callbacks.action` stay coupled:
  actions imply the callback family, and the callback family is not valid without declared actions.
- Rejected actions are mutation-free by API shape in the SDK:
  `ActionResponse::rejected()` can be finished, but it does not expose
  authoritative-mutation builder methods.
- Action callbacks require a real decoded `ActionInput`:
  empty or malformed action payload bytes are not silently synthesized by the
  SDK. On the runtime path, that becomes a contract / transport / host-delivery
  fault for the guest call rather than a fabricated placeholder input.
- Runtime messaging is a dedicated callback family on both sides rather than
  being stuffed into lifecycle or actions.
- Runtime-loaded guests use explicit world runtime services for queries,
  client visibility, world session state, client control, character physics,
  and observability rather than callback-specific hacks.
- Runtime delivery is contract-checked symmetrically:
  undeclared inbound channels/message ids fault the guest the same way undeclared outbound use does.
- Declarations now cover blocks, components, messages, worldgen,
  character-controllers, client-control-providers, channels, actions, and
  capability keys in one transport-neutral registration model.
- Worldgen output uses the same canonical terrain-write model as builtin
  worldgen: `WorldGenOutput.writes` plus
  `WorldGenOutput.bootstrap.initial_world_spawn_hint` for advisory initial
  world bootstrap spawn selection and
  `WorldTerrainWrite::{FillSection, FillBox, SetBlock}`.
- Those `WorldGen*` structures are owned by `freven_volumetric_api`; this SDK
  merely re-exports them so world-layer guests can author against the same
  volumetric contract as builtin providers.
- Block/content registration stays on `BlockDescriptor` and `BlockRuntimeId`;
  raw section encodings are not the authoring contract.
- Guest start callbacks receive `StartInput { session, experience_id, mod_id, config }`.
  `StartInputExt::config_typed::<T>()` decodes the canonical per-mod TOML
  config document for the guest path.
- `StartInput.session` is the canonical runtime-session identity for that guest
  instance on one hosted side. Stateful guests should key long-lived state off
  that session identity instead of ad hoc process statics.
- Capability declarations are validated honestly by the runtime:
  empty keys fail, and unknown capability keys are rejected against host policy.
- Provider families use the same canonical declaration model as builtin mods.
  The public `wasm_guest!` / `stateful_wasm_guest!` path now authors and exports
  `worldgen`, `character_controllers`, and `client_control_providers` without
  low-level ABI glue; side-specific hosting still follows the canonical runtime
  side rules.
- Stateful guest authoring now has an explicit session model through
  `StatefulGuestModule` / `stateful_wasm_guest!`: the SDK owns a per-runtime-session
  state slot, reuses it across callbacks in that session, and rotates it when a
  new `StartInput.session` arrives.
- Wasm is the primary safe path. Native and external transports remain
  secondary transport integrations with separate operational tradeoffs.
