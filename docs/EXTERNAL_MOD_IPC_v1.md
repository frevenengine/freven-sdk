# External Mod IPC v1

This document defines the companion-process protocol for mods with
`execution = "external_guest"`.

The canonical public guest contract is `freven_guest` as documented in
`GUEST_CONTRACT_v1.md`. External is a secondary transport that carries the same
guest negotiation and action semantics over a JSON envelope.

This is a secondary transport integration, not the default authoring story.
Prefer Wasm with `freven_guest_sdk` unless you specifically need a companion
process boundary.

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
- External transport supports the full `freven_guest` surface; if the guest
  declares a lifecycle hook, the companion process must answer the
  corresponding request with a `lifecycle` response carrying `LifecycleResult`.
- A guest callback may emit one or more `service_request` envelopes before it
  emits its terminal `lifecycle` / `handle_action` / `client_messages` /
  `server_messages` response.
- The host answers each `service_request` with a matching `service_response`
  using the same envelope `id`, then continues waiting for the terminal
  callback response.
- If a companion process exits/crashes, disconnects, violates protocol, or times out:
  - that mod is disabled for the current runtime session
  - later lifecycle callbacks stop
  - action calls for that mod return `ActionOutcome::Rejected`
  - host kills/waits child if still alive
- If a valid `ActionResult` cannot be completed because host-side runtime-command
  application fails, that still counts as a guest session fault:
  - the mod is disabled for the current runtime session
  - follow-up lifecycle/action calls are rejected
  - host kills/waits the companion child
- External mods are loaded only when explicit policy is enabled (for example `--allow-external-mods` or `FREVEN_ALLOW_EXTERNAL_MODS=1`).
