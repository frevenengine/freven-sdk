# Neutral Guest Contract v1

`freven_guest` is the canonical public contract for runtime-loaded neutral
Freven guests.

Most authors should consume this contract through `freven_guest_sdk`. Writing
directly against `freven_guest` is mainly for low-level contract tests,
runtime work, and adapter implementations.

Builtin / compile-time neutral authoring uses `freven_mod_api`, which is a
facade over the same semantic model.

The transport docs in this folder that describe Wasm, native, or external
execution currently document the explicit world-owned `freven_world_guest`
path. They are not the canonical reference for `freven_guest`.

## Scope

- Semantic contract only.
- Transport-agnostic.
- Shared meaning for neutral runtime-loaded guests.

## Versioning

- Contract version constant: `GUEST_CONTRACT_VERSION_1`
- Types in `freven_guest` are intentionally unversioned.
- Future breaking contract revisions should use a new contract version rather
  than mixing `vN` contract names with `*V1` type names inside the same family.

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

- `components`
- `messages`
- `channels`
- `capabilities`

`LifecycleHooks` currently exposes:

- `start_client`
- `start_server`
- `tick_client`
- `tick_server`

`MessageHooks` currently exposes:

- `client`
- `server`

World-shaped declarations such as blocks, actions, provider families, and
runtime world services are intentionally out of scope here. Those live in the
explicit world-owned `freven_world_guest` contract.

## Session identity

`RuntimeSessionInfo` is the canonical runtime-session identity for one hosted
guest side.

- `id: u64`
- `side: RuntimeSessionSide`

`RuntimeSessionSide` is one of:

- `client`
- `server`

Stateful neutral guests should key long-lived guest state off that session
identity instead of process-global statics.

## Observability

Neutral guest logging uses the shared observability payloads exported by
`freven_guest`:

- `LogLevel`
- `LogPayload`

`freven_guest_sdk` exposes host-routed log macros over that same contract.
