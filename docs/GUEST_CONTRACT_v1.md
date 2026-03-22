# Guest Contract v1

`freven_world_guest` is the canonical public contract for runtime-loaded Freven
world-stack mods.

Most mod authors should consume this contract through
`freven_world_guest_sdk`. Writing directly against `freven_world_guest` is
mainly for low-level transport work, fixtures, and runtime validation.

Builtin / compile-time world authoring uses `freven_world_api`, which is a
facade over the same semantic registration and runtime-output model.

Ownership note:

- `freven_world_guest` owns the generic runtime-loaded world contract and its
  runtime-service / runtime-output envelopes
- `freven_block_guest` owns runtime-loaded block mutation/query/service payload
  shapes
- block-owned families may be carried inside `freven_world_guest` envelopes,
  but that carrier role does not transfer block ownership to
  `freven_world_guest`

## Scope

- Semantic contract only.
- Transport-agnostic.
- Shared meaning across Wasm, native, and external-process backends.

Transport adapters may use different wire shapes, but they must carry the same
contract meaning defined here.

## Versioning

- Contract version constant: `GUEST_CONTRACT_VERSION_1`
- Types in `freven_world_guest` are intentionally unversioned.
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
  model across builtin, Wasm, native, and external guests
- the recommended public Wasm SDK path (`freven_world_guest_sdk::wasm_guest!` /
  `stateful_wasm_guest!`) now authors those provider families directly from the
  canonical registration surface, rather than through low-level export glue
- capability keys must be non-empty
- declared capability keys must exist in the resolved host capability table

Current hosting policy:

- builtin / compile-time authoring through `freven_world_api` hosts all currently
  implemented declaration families
- Wasm, native, and external guest transports host provider families
  (`worldgen`, `character_controllers`, `client_control_providers`) through the
  same canonical registration model used by builtin mods
- side-specific hosting is explicit:
  `worldgen` is hosted on server runtime sessions,
  `client_control_providers` are hosted on client runtime sessions,
  `character_controllers` are hosted on both sides
- external-process execution may still be blocked by explicit trust/policy
  settings such as `allow_external_mods`, but provider families are no longer a
  separate external-only semantic exception

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

Guest/runtime-loaded mods use explicit runtime service families through the
generic world runtime-service envelope:

- `WorldServiceRequest::Block(...)`
- `WorldServiceRequest::Query(...)`
- `WorldServiceRequest::ClientVisibility(...)`
- `WorldServiceRequest::Session(...)`
- `WorldServiceRequest::ClientControl(...)`
- `WorldServiceRequest::CharacterPhysics(...)`
- `WorldServiceRequest::Observability(...)`
- `RuntimeOutput.messages`
- `RuntimeOutput.blocks`

Ownership inside that model is explicit:

- `freven_world_guest` owns the generic runtime-service and runtime-output
  envelopes
- `freven_block_guest` owns block mutation/query/service payload shapes
- `WorldServiceRequest::Block(...)` / `WorldServiceResponse::Block(...)` and
  `RuntimeOutput.blocks` are carrier/composition points for those block-owned
  families
- that carrier role does not make `freven_world_guest` the owner of block
  gameplay semantics

Current query/session/visibility requests include:

- world/block reads
- player position reads
- player display-name reads
- player-to-entity resolution
- entity component-byte reads
- client player visibility reads
- client active level
- client next input sequence
- server player-connected checks

Current client-control/character-physics requests include:

- key and mouse-button ownership/bind queries
- key and mouse-button pressed-state queries
- mouse-delta and view-angle reads
- solid-world collision checks
- AABB sweeps and terrain movement resolution

Observability is a canonical semantic family owned by the host/runtime.
In contract v1, observability currently contains only logging:

- `RuntimeObservabilityRequest::Log(LogPayload)`
- `LogPayload.level`
- `LogPayload.message`

Canonical log levels are:

- `debug`
- `info`
- `warn`
- `error`

Logging is intentionally outside gameplay/result/effect semantics:

- it is fire-and-forget from the guest perspective
- it is not part of `ActionResult`
- it is not part of `LifecycleResult`
- it is not part of canonical message output
- sink failures, filtering, or suppression do not become gameplay protocol

The canonical guest log payload is intentionally minimal:

- severity/level
- UTF-8 message text

Guests do not define custom categories, arbitrary key/value fields, trace/span
ids, or sink selection in this phase. The host/runtime owns attribution,
formatting, routing, filtering, truncation, and final presentation.

Current block-mutation family carried by runtime output includes:

- `RuntimeOutput.blocks`
- `BlockMutationBatch.mutations`
- `BlockMutation::SetBlock { pos, block_id, expected_old }`

Current worldgen output family uses:

- `WorldGenOutput.writes`
- `WorldTerrainWrite::FillSection { sy, block_id }`
- `WorldTerrainWrite::FillBox { min, max, block_id }`
- `WorldTerrainWrite::SetBlock { pos, block_id }`

Transport adapters must carry these semantic families unchanged. They must not
invent transport-specific truth about reads, messages, or command application.
The same rule applies to observability/logging.

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

Observability/logging is also session-bound:

- log capability exists only while the guest instance is bound to the active
  runtime session
- after disable-for-session, later log emissions from that guest/session must
  be rejected or ignored by the host
- after unload, detach, hot reload, world reload, or reattach, the old session
  binding must no longer be accepted for further logs
- transport carriers must not keep stale log handles/channels alive after
  session end

For every accepted guest log record, the host/runtime enriches the record with
runtime-owned attribution where available:

- mod/guest identity
- execution kind
- side
- runtime session identity
- runtime source/artifact/trust/policy context
- active callback family or phase when honestly available

Guests must not be required to prefix messages manually with their own runtime
identity/context.

If a guest violates the contract or faults during a runtime session:

- that guest is disabled for the remainder of the runtime session
- further action dispatches to that guest must reject
- the host must stop routing later lifecycle and message callbacks to that guest for that
  session
- provider wrappers and other runtime-owned adapters must also stop invoking the
  guest for that session, even if the wrapper object itself still exists locally

For action callbacks, "faults" include host-side failure to apply the guest's
declared block mutation batch after the `ActionResult` is decoded and
validated.

For message callbacks, faults include invalid inbound scope mapping and
outbound sends that violate the negotiated channel/message contract.

For observability/logging, ordinary host policy outcomes are not session faults:

- oversized messages may be truncated
- debug visibility may be suppressed
- rate-limited records may be dropped or summarized

True boundary violations in the logging path are session faults:

- malformed transport envelopes
- invalid enum or impossible payload shapes
- adapter/bridge misuse that breaks the logging contract
