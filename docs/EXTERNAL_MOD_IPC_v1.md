# External Mod IPC v1

This document defines the companion-process protocol for mods with
`execution = "external_guest"`.

The canonical public guest contract for the current world stack is
`freven_world_guest` as documented in `GUEST_CONTRACT_v1.md`. External is a
secondary execution path that carries the same guest negotiation, lifecycle,
action, message, provider, and runtime service semantics over a JSON envelope.

This is a secondary transport integration, not the default authoring story.
Prefer Wasm with `freven_world_guest_sdk` unless you specifically need a
companion process boundary. Builtin world mods use the same semantic system
through `freven_world_api`, but they do not use this IPC transport.

## Transport

- Parent process spawns one OS process per external mod.
- IPC is line-delimited JSON over child `stdin`/`stdout` (UTF-8, one JSON object per line).
- Envelope fields:
  - `v`: protocol version (`1`)
  - `id`: request/response correlation id
  - `type`: message kind

## Requests

- `handshake`
  - payload: `host_version: u32`
  - required first call after spawn
- `negotiate`
  - payload: `request: NegotiationRequest`
- `service_response`
  - payload: `response: RuntimeServiceResponse`
- `start_client`
  - payload: `input: StartInput`
- `start_server`
  - payload: `input: StartInput`
- `tick_client`
  - payload: `input: TickInput`
- `tick_server`
  - payload: `input: TickInput`
- `handle_action`
  - payload: `input: ActionInput`
- `client_messages`
  - payload: `input: ClientMessageInput`
- `server_messages`
  - payload: `input: ServerMessageInput`
- `generate_worldgen`
  - payload: `input: WorldGenCallInput`
- `init_character_controller`
  - payload: `input: CharacterControllerInitInput`
- `step_character_controller`
  - payload: `input: CharacterControllerStepInput`
- `sample_client_control_provider`
  - payload: `input: ClientControlSampleInput`
- `shutdown`
  - best-effort clean shutdown request sent by host before process kill fallback

## Responses

- `handshake`
  - payload: `protocol_version: u32`
- `negotiate`
  - payload: `response: NegotiationResponse`
- `service_request`
  - payload: `request: RuntimeServiceRequest`
- `lifecycle`
  - payload: `result: LifecycleResult`
- `handle_action`
  - payload: `result: ActionResult`
- `client_messages`
  - payload: `result: ClientMessageResult`
- `server_messages`
  - payload: `result: ServerMessageResult`
- `generate_worldgen`
  - payload: `result: WorldGenCallResult`
- `init_character_controller`
  - payload: `result: CharacterControllerInitResult`
- `step_character_controller`
  - payload: `result: CharacterControllerStepResult`
- `sample_client_control_provider`
  - payload: `result: ClientControlSampleResult`
- `error`
  - payload: `message: String`

## Behavioral rules

`StartInput` carries `session`, `experience_id`, `mod_id`, and the resolved
per-mod config document (`ModConfigDocument`, currently TOML text).

- Host enforces per-call timeout for handshake, negotiation, steady-state
  lifecycle calls, and action IPC.
- Negotiation must select `GUEST_CONTRACT_VERSION_1` and return a
  `guest_id` that matches the resolved mod id.
- Negotiated lifecycle declarations may include both client and server hooks.
  The runtime hosts the active side as a subset for the current session.
- External transport supports the full `freven_world_guest` callback surface:
  lifecycle, action, message, and provider families all use the same canonical
  declaration model as builtin/Wasm/native guests.
- Side-specific hosting matches the canonical runtime model:
  `generate_worldgen` is issued only on server runtime sessions,
  `sample_client_control_provider` only on client runtime sessions,
  `init_character_controller` / `step_character_controller` on either side when
  that provider family is hosted there.
- If the guest declares a lifecycle hook, the companion process must answer the
  corresponding request with a `lifecycle` response carrying `LifecycleResult`.
- If the guest declares a provider callback family, the companion process must
  answer the corresponding provider request with the matching provider result
  envelope. Declared provider families must not be left operationally dead.
- A guest callback may emit one or more `service_request` envelopes before it
  emits its terminal `lifecycle` / `handle_action` / `client_messages` /
  `server_messages` / provider response.
- The host answers each `service_request` with a matching `service_response`
  using the same envelope `id`, then continues waiting for the terminal
  callback response.
- Logging for external guests is not `stderr`-based. It uses the same explicit
  `service_request` path as other canonical runtime services.
- Provider callbacks share the same runtime session lifetime as lifecycle,
  action, and message callbacks. A provider timeout, decode failure, protocol
  violation, invalid result, or runtime-service misuse disables that guest for
  the current runtime session and later callbacks/actions are rejected.
- If a companion process exits/crashes, disconnects, violates protocol, or times out:
  - that mod is disabled for the current runtime session
  - later lifecycle callbacks stop
  - later provider callbacks fault/reject through the disabled session
  - action calls for that mod return `ActionOutcome::Rejected`
  - host kills/waits child if still alive
- If a valid `ActionResult` cannot be completed because host-side runtime-command
  application fails, that still counts as a guest session fault:
  - the mod is disabled for the current runtime session
  - follow-up lifecycle/action calls are rejected
  - host kills/waits the companion child
- External mods are loaded only when explicit policy is enabled (for example `--allow-external-mods` or `FREVEN_ALLOW_EXTERNAL_MODS=1`).

## Observability / logging

External guests emit logs through the canonical service channel:

- response envelope: `service_request`
- request payload: `RuntimeServiceRequest::Observability(RuntimeObservabilityRequest::Log(LogPayload))`
- canonical payload: `LogPayload { level, message }`
- levels: `debug`, `info`, `warn`, `error`

The process boundary does not create separate logging semantics. External mods
still provide only level and message text. The host/runtime owns attribution,
policy, filtering, sanitization, truncation, rate limiting, routing, and final
presentation.

Every accepted external log is enriched host-side where available with mod
identity, execution kind (`external`), side, runtime session id, source,
artifact, trust, policy, and active callback family.

Session enforcement is mandatory:

- log ingestion is valid only while the active runtime session is alive
- after disable-for-session, detach, restart, unload, hot reload, world reload,
  or reattach, later logs from the old binding must be ignored/rejected
- host teardown must close the effective ingestion path so the old process
  cannot keep producing semantically live logs after session end

Fault policy distinguishes policy outcomes from boundary violations:

- oversized messages may be truncated safely
- spam may be dropped or summarized by host policy
- debug logs may be hidden by host policy
- malformed JSON/envelopes, impossible logging payloads, or protocol misuse are
  contract violations that disable the guest for the current runtime session
