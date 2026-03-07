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

Host and guest negotiate before any lifecycle or action callback.

- `NegotiationRequest`
  - `supported_contract_versions: Vec<u32>`
  - `transport: GuestTransport`
- `NegotiationResponse`
  - `selected_contract_version: u32`
  - `description: GuestDescription`

## Guest description

`GuestDescription` declares:

- `guest_id`
- `lifecycle: LifecycleHooks`
- `action_entrypoint`
- `actions: Vec<ActionBinding>`

`LifecycleHooks` currently exposes:

- `start_client`
- `start_server`
- `tick_client`
- `tick_server`

`on_start_common` is intentionally not part of the guest contract yet.

## Action path

- Host sends `ActionInput`
- Guest returns `ActionResult`
- `ActionResult.outcome` is `applied` or `rejected`
- `ActionResult.effects` currently supports world effects through `WorldEffect`

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
- the host must stop routing later lifecycle callbacks to that guest for that
  session

For action callbacks, "faults" include host-side failure to apply the guest's
declared world effects after the `ActionResult` is decoded and validated.
