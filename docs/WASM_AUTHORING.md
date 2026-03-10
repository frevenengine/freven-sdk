# Wasm Authoring

This is the recommended public path for Freven runtime-loaded mods.

Most mod authors should write against `freven_guest_sdk`, not hand-roll the raw
Wasm ABI exports.

## Why this is the default path

- Wasm is the primary safe guest transport.
- `freven_guest_sdk` keeps the canonical `freven_guest` lifecycle and action
  model visible while hiding export-table, allocation, and `postcard` plumbing.
- Raw ABI work is still available for fixtures and runtime validation, but it is
  not the normal getting-started experience.
- Builtin / compile-time mods use `freven_mod_api`, but they still participate
  in the same semantic registration and runtime-output model.

## Canonical lifecycle in guest contract v1

The canonical guest lifecycle today is:

- negotiation
- `start_client`
- `start_server`
- `tick_client`
- `tick_server`
- client message callbacks
- server message callbacks
- action handling through one action entrypoint plus declared bindings

Current contract shape:

- lifecycle hooks return `LifecycleResult`
- action callbacks return `ActionResult`
- message callbacks return `ClientMessageResult` / `ServerMessageResult`
- all three callback families emit the same `RuntimeOutput` families
- `on_start_common` is not part of the runtime-loaded guest contract

Those boundaries are intentional. The public story is one semantic model, but
the transport references remain transport-specific where the encoding actually
differs.

## Minimal authoring example

```rust
use freven_guest_sdk::{ActionContext, ActionResponse};

const PLACE_BLOCK: u32 = 1;

fn handle_action(ctx: ActionContext<'_>) -> ActionResponse {
    let _ = ctx.player_id();
    ActionResponse::applied().set_block((4, 80, 4), 1)
}

freven_guest_sdk::wasm_guest!(
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
- negotiation/lifecycle/action export implementation details
- `postcard` encode/decode of contract payloads
- packed `(ptr, len)` return wiring
- action binding dispatch by `binding_id`

What stays explicit:

- guest id
- declared registration families
- declared lifecycle hooks
- declared action bindings
- exported Wasm capability surface generated from that same declaration
- canonical runtime output semantics and authoritative world commands

`wasm_guest!` is intentionally declarative rather than magical. The lifecycle
hooks and action bindings you write are the same data used to build the
canonical `GuestDescription` and to emit the Wasm export table, so the two
surfaces cannot drift in normal authoring.

The canonical registration model includes blocks, components, messages,
channels, actions, capabilities, worldgen keys, character-controller keys, and
client-control-provider keys. Wasm guests may declare provider families, and
the runtime now hosts them through the same semantic registration model used by
builtin, native, and external execution. Side and policy still gate which
providers are actually hosted in a given runtime session.

`GuestModule` plus `export_wasm_guest!(...)` still exist as a lower-level escape
hatch for raw ABI fixtures, runtime validation, or unusual tests, but they are
not the recommended public authoring path.

## Payload ergonomics

`ActionContext` exposes the canonical input fields directly:

- `binding_id()`
- `player_id()`
- `level_id()`
- `stream_epoch()`
- `action_seq()`
- `at_input_seq()`
- `payload()`
- `decode_payload::<T>()`

Use `ActionResponse::applied()` or `ActionResponse::rejected()` to surface the
canonical outcome, then attach runtime output such as `.set_block(...)` or
message sends.

Two SDK hardening rules matter here:

- `ActionResponse::rejected()` is terminal at the API level:
  the rejected response builder can be finished, but it does not expose
  authoritative-command builder methods.
- Action callbacks require a real decoded `ActionInput`; empty or malformed
  action payload bytes are not silently synthesized by the SDK.
  In practice this means a contract / transport / host-delivery violation on
  the action callback path faults the guest call instead of fabricating a
  placeholder input.

## Start-time config semantics

`StartInput` now carries:

- `experience_id`
- `mod_id`
- `config`

The config document is the resolved per-mod `experience.config."<mod_id>"`
table serialized as TOML text. `freven_guest_sdk::StartInputExt` exposes
`config_text()` and `config_typed::<T>()` helpers so guest authors can read the
same per-mod config semantics builtin / compile-time mods already had.

## Runtime services

`freven_guest_sdk` exposes `RuntimeServices` for runtime-loaded guests:

- reads: `block_world`, `player_position`, `player_display_name`,
  `player_entity_id`, `entity_component_bytes`
- side-specific facilities: `client_active_level`, `client_next_input_seq`,
  `server_player_connected`

These calls are semantic runtime services. They are not ad-hoc callback hacks
and they are not encoded as fake action results.

## Logging

Use the log macros from `freven_guest_sdk` to emit messages through the
canonical observability service:
```rust
use freven_guest_sdk::{log_debug, log_info, log_warn, log_error};

// inside any lifecycle, action, or message callback:
log_info!("player {} joined the world", player_id);
log_warn!("block at {:?} was already air", pos);
```

All four levels are available: `log_debug!`, `log_info!`, `log_warn!`,
`log_error!`. They accept the same format syntax as `format!`.

Logging is fire-and-forget: it does not affect `ActionResult`,
`LifecycleResult`, or message output. The host owns attribution, filtering,
routing, and presentation — guests provide only level and message text.

The macros call `RuntimeServices.log(...)` under the hood, which routes through
`RuntimeServiceRequest::Observability`. On Wasm this goes through
`freven_guest_host_service_call`; on native it goes through
`NativeRuntimeBridge`. The guest code is identical across transports.

## Transport guidance

Prefer these paths in this order:

1. Wasm via `freven_guest_sdk`
2. External process integration when you explicitly need process isolation
3. Native only for trusted local code and engine/runtime development

Native and external paths are secondary today. They remain important for
specific cases, but they should not be presented as equivalent onboarding paths
or as safer alternatives to Wasm.

Builtin / compile-time execution is the same semantic system through a
different execution path. It is not a transport rival to Wasm; use
`freven_mod_api` when you are authoring code that ships builtin with the
experience or host.

## Reference docs

- Canonical contract: [GUEST_CONTRACT_v1.md](GUEST_CONTRACT_v1.md)
- Wasm transport reference: [WASM_ABI_v1.md](WASM_ABI_v1.md)
- Native transport reference: [NATIVE_MOD_ABI_v1.md](NATIVE_MOD_ABI_v1.md)
- External transport reference: [EXTERNAL_MOD_IPC_v1.md](EXTERNAL_MOD_IPC_v1.md)
