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
- Wasm and native guest transports host provider families
  (`worldgen`, `character_controllers`, `client_control_providers`) through the
  same canonical registration model used by builtin mods
- external-process guest execution still policy-gates provider families
  explicitly because that transport does not yet expose safe provider hosting
- this is an execution/policy gate, not a separate public declaration model

## Action path

- Host sends `ActionInput`
- Guest returns `ActionResult`
- `ActionResult.outcome` is `applied` or `rejected`
- `ActionResult.output` carries canonical runtime output families
- rejected actions may carry message output, but must not carry command output
- `ActionInput.player_position_m` remains an action-scoped convenience slice, not the runtime service model

## Message path

- Host sends `ClientMessageInput` / `ServerMessageInput`
- Guest returns `ClientMessageResult` / `ServerMessageResult`
- Messaging is one canonical runtime-output family (`RuntimeOutput.messages`)
- lifecycle, action, and message callbacks all use the same message semantics
- outbound sends must use declared message ids and declared side-appropriate writable channels
- inbound delivery is routed only for declared side-appropriate readable channels and declared message ids
- unsupported/unknown message scope mapping is a guest fault, not a silent fallback

## Lifecycle path

- Host sends `StartInput` or `TickInput`
- Guest returns `LifecycleResult`
- `LifecycleResult.output` uses the same canonical runtime output families as actions and message callbacks

`StartInput` carries:

- `session`
- `experience_id`
- `mod_id`
- `config`

`config` is the resolved per-mod config document from `experience.config."<mod_id>"`.
Contract v1 currently serializes that document as TOML text with an explicit
`ModConfigFormat`.

Runtime/config/experience metadata is carried where it is semantically stable:

- `StartInput.session`
- `StartInput.experience_id`
- `StartInput.mod_id`
- `StartInput.config`
- `TickInput.tick`
- `TickInput.dt_millis`

There is intentionally no separate lifecycle-only side channel.

## Runtime services

Guest/runtime-loaded mods now use explicit runtime service families:

- `RuntimeServiceRequest::Read(...)`
- `RuntimeServiceRequest::Side(...)`
- `RuntimeOutput.messages`
- `RuntimeOutput.commands`

Current read requests include:

- world/block reads
- player position reads
- player display-name reads
- player-to-entity resolution
- entity component-byte reads

Current side-specific requests include:

- client active level
- client next input sequence
- server player-connected checks

Current command families include:

- `RuntimeCommandOutput.world`
- `WorldCommand::SetBlock { pos, block_id, expected_old }`

Transport adapters must carry these semantic families unchanged. They must not
invent transport-specific truth about reads, messages, or command application.

## Disable-on-session semantics

`StartInput.session` is the canonical runtime-session identity for one resolved
guest mod on one hosted side.

A runtime session begins when the host attaches that guest for a side, accepts
negotiation, assigns the session id, and later delivers the matching
`start_client` or `start_server` callback.

A runtime session ends when that hosted side unloads, hot-reloads, world-reloads
through runtime reconstruction, detaches, is reattached as a fresh host session,
or the hosting process exits. Reconnect by itself is not a semantic session
boundary unless it rebuilds the hosted guest runtime.

Guest SDKs may keep per-session state, but that state is scoped to the
`StartInput.session` identity and must be discarded when a new session id is
started.

If a guest violates the contract or faults during a runtime session:

- that guest is disabled for the remainder of the runtime session
- further action dispatches to that guest must reject
- the host must stop routing later lifecycle and message callbacks to that guest for that
  session
- provider wrappers and other runtime-owned adapters must also stop invoking the
  guest for that session, even if the wrapper object itself still exists locally

For action callbacks, "faults" include host-side failure to apply the guest's
declared runtime commands after the `ActionResult` is decoded and validated.

For message callbacks, faults include invalid inbound scope mapping and
outbound sends that violate the negotiated channel/message contract.
