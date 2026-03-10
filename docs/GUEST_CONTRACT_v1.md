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
- `worldgen`
- `character_controllers`
- `client_control_providers`
- `channels`
- `actions`
- `capabilities`

`LifecycleHooks` currently exposes:

- `start_client`
- `start_server`
- `tick_client`
- `tick_server`

`MessageHooks` currently exposes:

- `client`
- `server`

`on_start_common` is not part of the guest contract.

Registration/callback invariants:

- `registration.actions` and `callbacks.action` are one family:
  declaring actions requires `callbacks.action = true`
- `callbacks.action = true` requires at least one declared action
- provider families (`worldgen`, `character_controllers`,
  `client_control_providers`) are part of the canonical public declaration
  model even when a given execution/policy class does not host them yet
- capability keys must be non-empty
- declared capability keys must exist in the resolved host capability table

Current hosting policy:

- compile-time/builtin registration hosts all currently implemented declaration
  families
- runtime-loaded guest transports may declare provider families canonically, but
  host policy currently rejects them explicitly because guest factory/runtime
  hosting for those families does not exist yet
- this is an execution/policy gate, not a separate public declaration model

## Action path

- Host sends `ActionInput`
- Guest returns `ActionResult`
- `ActionResult.outcome` is `applied` or `rejected`
- `ActionResult.effects` currently supports world effects through `WorldEffect`
- `ActionInput.player_position_m` is the first canonical player-read slice

## Message path

- Host sends `ClientMessageInput` / `ServerMessageInput`
- Guest returns `ClientMessageResult` / `ServerMessageResult`
- Messaging is a dedicated callback family, separate from lifecycle and actions
- outbound sends must use declared message ids and declared side-appropriate writable channels
- inbound delivery is routed only for declared side-appropriate readable channels and declared message ids
- unsupported/unknown message scope mapping is a guest fault, not a silent fallback

## Lifecycle path

Lifecycle calls are currently ack-only.

- Host sends `StartInput` or `TickInput`
- Guest returns `LifecycleAck`

`StartInput` carries:

- `experience_id`
- `mod_id`
- `config`

`config` is the resolved per-mod config document from `experience.config."<mod_id>"`.
Contract v1 currently serializes that document as TOML text with an explicit
`ModConfigFormat`.

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

For message callbacks, faults include invalid inbound scope mapping and
outbound sends that violate the negotiated channel/message contract.
