# External Mod IPC v1

This document defines the companion-process protocol for `kind = "external"` mods.

This is a legacy action-only transport protocol. The canonical public guest
contract is `freven_guest` as documented in `GUEST_CONTRACT_v1.md`.

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
- `init_manifest`
  - returns the mod manifest/action bindings
- `handle_action`
  - payload:
    - `kind: u32` (mod-local action id from manifest)
    - `player_id: u64`
    - `at_input_seq: u32`
    - `payload: Vec<u8>`
- `shutdown`
  - best-effort clean shutdown request sent by host before process kill fallback

## Responses

- `handshake`
  - payload: `protocol_version: u32`
- `init_manifest`
  - payload: `manifest: ModManifestV1`
- `handle_action`
  - payload: `result: ActionResultV1`
- `error`
  - payload: `message: String`

## Behavioral rules

- Host enforces per-call timeout for handshake/init/action IPC.
- If a companion process exits/crashes, disconnects, violates protocol, or times out:
  - that mod is disabled for the current runtime session
  - action calls for that mod return `ActionOutcome::Rejected`
  - host kills/waits child if still alive
- External mods are loaded only when explicit policy is enabled (`allow_external_mods`).
