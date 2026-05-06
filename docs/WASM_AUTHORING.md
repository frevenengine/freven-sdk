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

## Client input contract

Client-control providers use a physical input surface, not text input.

- `ClientKeyCode` follows W3C `KeyboardEvent.code` / winit-style physical-key names where practical.
- Use `Digit1`..`Digit9` for hotbars and number-row gameplay bindings.
- Use `KeyA`..`KeyZ` for physical letter-key locations; legends may differ on AZERTY/QWERTZ/etc.
- `Shift` and `Ctrl` remain compatibility aggregates for older code, but new bindings should prefer `ShiftLeft` / `ShiftRight` and `ControlLeft` / `ControlRight`.
- `ClientMouseButton::{Back, Forward, Other(u16)}` supports extra mouse buttons beyond left/right/middle.

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


## Disk layout and authored wiring

There are two common disk layouts for runtime-loaded Wasm mods.

### Instance-local mod in a mutable install

Use this when you drop a mod into an install under `<instance>/mods/<mod_id>/...`.

```text
<instance>/
  experiences/<experience_id>/experience.toml
  mods/example.hello/
    mod.toml
    mod.wasm
```

Minimal `mod.toml`:

```toml
schema = 3
id = "example.hello"
version = "0.1.0"
artifact = "wasm_module"
execution = "wasm_guest"
trust = "sandboxed"
policy = "safe_guest"
surfaces = "both"
entry = "mod.wasm"
```

Reference it from the active experience by id/version:

```toml
[[mods]]
id = "example.hello"
version = "^0.1"
```

Runtime config belongs in the active experience:

```toml
[config."example.hello"]
greeting = "hello"
tick_every = 1
```

### Bundled product-owned mod inside an experience

Use this when a bundled/shipped experience owns the mod subtree itself.

```text
<experience_root>/
  experience.toml
  mods/example.standalone.shell.core/
    mod.toml
    mod.wasm
```

Reference it with an explicit relative manifest path:

```toml
[[mods]]
id = "example.standalone.shell.core"
version = "^0.1"
path = "mods/example.standalone.shell.core/mod.toml"
```

Notes:
- `mod.toml` is the manifest / capability-request surface, not the active runtime config document.
- Guest `StartInput.config` comes from `experience.config."<mod_id>"`.
- Declare `[capabilities]` only when you need non-default limits or worldgen-specific budgets.
- Use `surfaces = "server"` for server-only mods.
  Use `surfaces = "both"` only when the guest is meant to attach on both sides.

## Capability requests

Declare Wasm capabilities in `mod.toml` as host policy requests, not as
arbitrary guest-controlled runtime knobs.

Current accepted capability keys are:

- default/hot callback profile:
  `max_linear_memory_bytes`, `max_call_millis`
- worldgen provider profile:
  `worldgen_max_linear_memory_bytes`, `worldgen_max_call_millis`,
  `worldgen_max_result_bytes`
- global validation flag:
  `allow_unstable`

Default/hot callback profile:

- applies to lifecycle, tick, action, message, character-controller, and
  client-control style interactive callbacks
- current policy:
  memory = `4 MiB`, call watchdog = `25 ms`, result bytes = `256 KiB`

Worldgen provider profile:

- applies only to declared worldgen providers
- current policy defaults:
  memory = `64 MiB`, call watchdog = `100 ms`, result bytes = `1 MiB`
- current policy maxima:
  memory = `128 MiB`, call watchdog = `250 ms`, result bytes = `4 MiB`

Rules that matter in practice:

- unknown capability keys are rejected
- invalid value types are rejected
- `allow_unstable` must remain `false`
- old `max_*` keys do not raise worldgen limits
- `worldgen_*` keys do not raise default/hot callback limits
- `worldgen_*` keys require a declared worldgen provider
- a both-side mod may declare `worldgen_*` keys and still attach on the client
  side; the client side just does not host the worldgen runner

Example:

```toml
[capabilities]
max_linear_memory_bytes = 4194304
max_call_millis = 25
worldgen_max_linear_memory_bytes = 67108864
worldgen_max_call_millis = 100
worldgen_max_result_bytes = 1048576
allow_unstable = false
```

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



### `WorldTerrainWrite::FillBox` bounds semantics

`WorldTerrainWrite::FillBox` uses half-open bounds in absolute world-cell
space: `[min, max)`.

Example vertical run for one `(x, z)` column:

```rust
use freven_world_guest_sdk::{WorldCellPos, WorldTerrainWrite};

let write = WorldTerrainWrite::FillBox {
    min: WorldCellPos::new(x, start_y, z),
    max: WorldCellPos::new(x + 1, end_y_exclusive, z + 1),
    block_id,
};
```

What this means:
- `min` is inclusive
- `max` is exclusive
- `min == max` is invalid because it produces zero volume
- minimum valid box extent is `1` on every axis
- coordinates are absolute world-cell positions, not section-local offsets

In practice:
- use `SetBlock` for sparse or isolated cells
- use `FillBox` for contiguous rectangular regions or vertical runs
- use `FillSection` when one full section is uniform

A vertical run at one `(x, z)` column therefore still needs:
- `max.x = min.x + 1`
- `max.z = min.z + 1`

Do not treat `max` as an inclusive last block coordinate.
If you have an inclusive end y from your own algorithm, convert it first to an
exclusive bound before emitting `FillBox`.


### Initial world spawn hints for custom worldgen providers

Custom worldgen providers may return an advisory initial bootstrap spawn hint
through `WorldGenOutput.bootstrap.initial_world_spawn_hint`.

Example:

```rust
use freven_world_guest_sdk::{
    InitialWorldSpawnHint,
    WorldGenBootstrapOutput,
    WorldGenOutput,
};

fn finish_worldgen(
    writes: Vec<freven_world_guest_sdk::WorldTerrainWrite>,
    surface_y: f32,
) -> WorldGenOutput {
    WorldGenOutput {
        writes,
        bootstrap: WorldGenBootstrapOutput {
            initial_world_spawn_hint: Some(InitialWorldSpawnHint {
                feet_position: [16.5, surface_y + 2.0, 16.5],
            }),
        },
    }
}
```

What this means:
- this is for **initial world bootstrap** only, not general respawn policy
- `feet_position` is a world-space **feet** position, not a collider center
- the host resolves authoritative initial spawn and may validate or adjust the
  final result against loaded terrain
- current bootstrap flow explicitly probes the bootstrap worldgen column and
  consumes the hint from that bootstrap bring-up path; later worldgen calls do
  not redefine runtime spawn policy

Recommended strategy:
- return a natural safe candidate from your terrain generator, such as a known
  walkable surface point with standing room
- prefer a plausible gameplay spawn area rather than flattening terrain around
  `(0, 0)` just to force safe spawning
- treat the hint as advice to bootstrap resolution, not as a guarantee that the
  exact returned feet position will be used unchanged

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
