# freven_guest_sdk

High-level guest authoring helpers for Freven runtime-loaded mods.

Use this crate for the normal Wasm authoring path. It sits on top of the
canonical `freven_guest` contract and hides the transport boilerplate:

- guest alloc/dealloc exports
- `postcard` encode/decode plumbing
- Wasm export table wiring
- lifecycle/action dispatch lookup
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

`wasm_guest!` is the normal public authoring path: the guest id, lifecycle
hooks, action bindings, negotiated `GuestDescription`, and emitted Wasm export
surface all come from that one declaration.

`GuestModule` plus `export_wasm_guest!(...)` remain available for lower-level
fixtures and ABI-focused tests when you intentionally need to wire the raw
surface yourself.

## Current boundaries

- Lifecycle hooks are ack-only because contract v1 does not expose lifecycle
  output payloads yet.
- Guest-side persistent instance state is not modeled by the SDK today. Use
  explicit statics only when you fully control the implications.
- Wasm is the primary safe path. Native and external transports remain
  secondary transport integrations with separate operational tradeoffs.
