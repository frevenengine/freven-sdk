# freven_guest_sdk

High-level guest authoring helpers for Freven runtime-loaded mods.

Use this crate for the normal Wasm authoring path. It sits on top of the
canonical `freven_guest` contract and hides the transport boilerplate:

- guest alloc/dealloc exports
- `postcard` encode/decode plumbing
- Wasm export table wiring
- native in-process export wiring for low-level fixtures/tests
- canonical declaration builders for blocks/components/messages/worldgen/character-controllers/client-control-providers/channels/actions/capabilities
- lifecycle/action/message dispatch lookup
- export-surface validation against the canonical `GuestDescription`

Most mod authors should depend on `freven_guest_sdk`. Reach for
`freven_guest` directly only when you are implementing or testing the raw guest
contract itself.

## Minimal example

```rust
use freven_guest_sdk::{ActionContext, ActionResponse};

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
            binding_id: 1,
            handler: handle_action,
        },
    },
);
```

`wasm_guest!` is the normal public authoring path: the guest id, registration
families, callback families, negotiated `GuestDescription`, and emitted Wasm
export surface all come from that one declaration.

`GuestModule` plus `export_wasm_guest!(...)` / `export_native_guest!(...)`
remain available for lower-level fixtures and ABI-focused tests when you
intentionally need to wire the raw surface yourself.

## Current boundaries

- Lifecycle hooks return `LifecycleResult`.
- `registration.actions` and `callbacks.action` stay coupled:
  actions imply the callback family, and the callback family is not valid without declared actions.
- Rejected actions are command-free by API shape in the SDK:
  `ActionResponse::rejected()` can be finished, but it does not expose
  authoritative-command builder methods.
- Action callbacks require a real decoded `ActionInput`:
  empty or malformed action payload bytes are not silently synthesized by the
  SDK. On the runtime path, that becomes a contract / transport / host-delivery
  fault for the guest call rather than a fabricated placeholder input.
- Runtime messaging is a dedicated callback family on both sides
  (`on_client_messages`, `on_server_messages`) rather than being stuffed into lifecycle or actions.
- Runtime-loaded guests use explicit runtime services for reads and side-specific
  facilities rather than callback-specific hacks.
- Runtime delivery is contract-checked symmetrically:
  undeclared inbound channels/message ids fault the guest the same way undeclared outbound use does.
- Declarations now cover blocks, components, messages, worldgen,
  character-controllers, client-control-providers, channels, actions, and
  capability keys in one transport-neutral registration model.
- Guest start callbacks receive `StartInput { experience_id, mod_id, config }`.
  `StartInputExt::config_typed::<T>()` decodes the canonical per-mod TOML
  config document for the guest path.
- Capability declarations are validated honestly by the runtime:
  empty keys fail, and unknown capability keys are rejected against host policy.
- Provider families use the same canonical declaration model as builtin mods.
  Wasm and native guests can now host `worldgen`, `character_controllers`, and
  `client_control_providers`; transports that cannot support a family for a
  given execution/policy class must still declare it canonically and are gated
  explicitly by host policy.
- Guest-side persistent instance state is not modeled by the SDK today. Use
  explicit statics only when you fully control the implications.
- Wasm is the primary safe path. Native and external transports remain
  secondary transport integrations with separate operational tradeoffs.
