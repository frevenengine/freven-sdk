# Wasm Authoring

This is the recommended public path for Freven runtime-loaded Wasm mods.

Current authoring is split into two explicit layers:

- `freven_guest_sdk`: neutral guest authoring for lifecycle, messages,
  components, channels, capabilities, session identity, and observability
- `freven_world_guest_sdk`: explicit world-stack authoring for block/content
  registration, action handlers, world queries and mutations, terrain-write
  worldgen, character controllers, client-control providers, and world runtime
  services

Most gameplay mods and current Vanilla-style authoring should use
`freven_world_guest_sdk`.

## Why Wasm remains the default path

- Wasm is the primary safe guest transport.
- The SDK crates keep the canonical guest contracts visible while hiding
  export-table, allocation, and `postcard` plumbing.
- Raw ABI work is still available for fixtures and runtime validation, but it
  is not the normal getting-started experience. The current ABI / IPC reference
  docs describe the explicit world-owned `freven_world_guest` path; neutral
  `freven_guest` reference material is split out separately.
- Builtin / compile-time mods use `freven_mod_api` or `freven_world_api`, but
  they still participate in the same semantic registration and runtime-output
  model.

## Choose the right surface

Use `freven_guest_sdk` when your guest needs only neutral platform-shaped
declarations:

- lifecycle hooks
- client/server message hooks
- generic components
- generic messages
- generic channels
- capabilities
- observability

Use `freven_world_guest_sdk` when your guest needs current world-stack
semantics:

- block / voxel content registration
- action handlers that read/query world state and emit world mutations
- worldgen that returns `WorldGenOutput.writes` terrain writes
- character controllers
- client-control providers
- world runtime services:
  `WorldServiceRequest::{Query, ClientVisibility, Session, ClientControl, CharacterPhysics, Observability}`
- player/world view queries and other world-facing runtime hooks

## Minimal neutral example

```rust
freven_guest_sdk::log_info!("hello from a neutral guest");
```

## Minimal world authoring example

```rust
use freven_world_guest_sdk::{ActionContext, ActionResponse};

const PLACE_BLOCK: u32 = 1;

fn handle_action(ctx: ActionContext<'_>) -> freven_world_guest_sdk::ActionResult {
    let _ = ctx.player_id();
    ActionResponse::applied().set_block((4, 80, 4), 1).finish()
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
            binding_id: PLACE_BLOCK,
            handler: handle_action,
        },
    },
);
```

What the SDK hides:

- `freven_guest_alloc` / `freven_guest_dealloc`
- negotiation and callback export implementation details
- `postcard` encode/decode of contract payloads
- packed `(ptr, len)` return wiring
- dispatch by declared binding or message/channel ids

What stays explicit:

- guest id
- declared registration families
- declared lifecycle hooks
- declared message or action bindings
- exported Wasm capability surface generated from that same declaration
- canonical runtime output semantics

`freven_world_guest_sdk::wasm_guest!` is intentionally declarative rather than
magical. The hooks and registrations you write are the same data used to build
the canonical `GuestDescription` and to emit the Wasm export surface.

## World authoring details

`freven_world_guest_sdk` exposes the current world-stack registration families
directly in `registration`:

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

It also owns the current world runtime helpers:

- `ActionContext`
- `ActionResponse`
- `StartInputExt`
- `RuntimeServices`
- `WorldGenOutput`
- `WorldTerrainWrite`
- `ClientMessageResponse`
- `ServerMessageResponse`

`WorldGenOutput`/`WorldTerrainWrite` come from the volumetric-owned
`freven_volumetric_api` crate; the world guest SDK simply re-exports them.
That includes `WorldGenOutput.bootstrap.initial_world_spawn_hint`, an advisory
initial world bootstrap feet-position hint rather than a generic respawn
policy.

Those surfaces are intentionally not available from the neutral
`freven_guest_sdk` crate.

## Logging

Both Wasm SDK layers expose log macros that emit through the canonical
observability service:

```rust
use freven_world_guest_sdk::{log_debug, log_error, log_info, log_warn};

log_info!("guest started");
log_warn!("falling back to default config");
```

Logging is fire-and-forget: it does not affect lifecycle/message/action output.
The host owns attribution, filtering, routing, and presentation.

## Transport guidance

Prefer these paths in this order:

1. Wasm via `freven_guest_sdk` or `freven_world_guest_sdk`
2. External process integration when you explicitly need process isolation
3. Native only for trusted local code and engine/runtime development

Builtin / compile-time execution is the same semantic system through a
different execution path. Use `freven_mod_api` for neutral builtin authoring
and `freven_world_api` for current world-stack builtin authoring.

## Reference docs

- Canonical neutral guest contract: [NEUTRAL_GUEST_CONTRACT_v1.md](NEUTRAL_GUEST_CONTRACT_v1.md)
- Canonical world-stack guest contract: [GUEST_CONTRACT_v1.md](GUEST_CONTRACT_v1.md)
- World-stack Wasm transport reference: [WASM_ABI_v1.md](WASM_ABI_v1.md)
- World-stack native transport reference: [NATIVE_MOD_ABI_v1.md](NATIVE_MOD_ABI_v1.md)
- World-stack external transport reference: [EXTERNAL_MOD_IPC_v1.md](EXTERNAL_MOD_IPC_v1.md)
