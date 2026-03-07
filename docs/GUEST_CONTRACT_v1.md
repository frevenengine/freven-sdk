# Guest Contract v1

`freven_guest` is the canonical public contract for runtime-loaded Freven mods.

Most mod authors should consume this contract through `freven_guest_sdk`.
Writing directly against `freven_guest` is mainly for low-level transport work,
fixtures, and runtime validation.

## Scope

- Semantic contract only.
- Transport-agnostic.
- Shared meaning across Wasm, native, and external-process backends.

Transport adapters may use different wire shapes, but they must carry the same
contract meaning defined here.

## Versioning

- Contract version constant: `GUEST_CONTRACT_VERSION_1`
- Types in `freven_guest` are intentionally unversioned.
- Future breaking contract revisions should use a new contract version and, if
  needed, a new module path or crate revision. Do not mix `contract vN` with
  `*V1` type names inside the same contract family.

## Negotiation

Host and guest negotiate before any guest callback.

- `NegotiationRequest`
  - `supported_contract_versions: Vec<u32>`
  - `transport: GuestTransport`
- `NegotiationResponse`
  - `selected_contract_version: u32`
  - `description: GuestDescription`

## Guest description

`GuestDescription` declares:

- `guest_id`
- `registration: GuestRegistration`
- `callbacks: GuestCallbacks`

`GuestRegistration` currently covers:

- `blocks`
- `components`
- `messages`
- `channels`
- `actions`
- `capabilities`

`LifecycleHooks` currently exposes:

- `start_client`
- `start_server`
- `tick_client`
- `tick_server`

`on_start_common` is intentionally not part of the guest contract yet.

Registration/callback invariants:

- `registration.actions` and `callbacks.action` are one family:
  declaring actions requires `callbacks.action = true`
- `callbacks.action = true` requires at least one declared action
- capability keys must be non-empty
- declared capability keys must exist in the resolved host capability table

## Action path

- Host sends `ActionInput`
- Guest returns `ActionResult`
- `ActionResult.outcome` is `applied` or `rejected`
- `ActionResult.effects` currently supports world effects through `WorldEffect`
- `ActionInput.player_position_m` is the first canonical player-read slice

## Server message path

- Host sends `ServerMessageInput`
- Guest returns `ServerMessageResult`
- This is a dedicated callback family, separate from actions and lifecycle
- `ServerMessageResult.outbound` carries `ServerOutboundMessage` sends
- host routing is channel/message-contract checked:
  inbound messages are delivered only for declared server-readable channels and declared message ids
- outbound sends must use declared message ids and declared server-writable channels
- unsupported/unknown message scope mapping is a guest fault, not a silent fallback

## Lifecycle path

Lifecycle calls are currently ack-only.

- Host sends `StartInput` or `TickInput`
- Guest returns `LifecycleAck`

There is intentionally no lifecycle effect/output channel in contract v1.
Lifecycle outputs are deferred until the runtime supports a real, honest
end-to-end lifecycle output model.

## Disable-on-session semantics

If a guest violates the contract or faults during a runtime session:

- that guest is disabled for the remainder of the runtime session
- further action dispatches to that guest must reject
- the host must stop routing later lifecycle and message callbacks to that guest for that
  session

For action callbacks, "faults" include host-side failure to apply the guest's
declared world effects after the `ActionResult` is decoded and validated.

For server-message callbacks, faults include invalid inbound scope mapping and
outbound sends that violate the negotiated channel/message contract.
