# World Async Service Model v1

This document defines the canonical guest-facing async/background computation
model for the world-stack guest contract family.

It is the public semantic truth for runtime-loaded world guests across Wasm,
native, and external transports.

This is a docs-first architecture document. It intentionally does not define
final Rust APIs, raw ABI structs, or implementation-specific scheduler
mechanics.

## Scope

- Semantic and architectural only.
- Transport-agnostic.
- Applies to guest-facing world async/background computation for
  `freven_world_guest`-based execution.
- Defines the canonical public model that transport adapters and SDK layers
  must preserve.

## Problem Statement

Some guest workloads are too expensive to complete on the authoritative owner
thread inside a single synchronous callback, but the platform still needs one
honest execution model across Wasm, native, and external guests.

Without a canonical async model, each transport would be tempted to invent its
own truth:

- native-only raw threads
- transport-specific callbacks
- ad hoc background job APIs
- hidden host push channels

That would break transport parity, blur host/runtime ownership, and leave
shutdown, backpressure, observability, and authoritative world-commit
semantics undefined.

The platform therefore needs one explicit public model for background
computation that preserves host ownership and remains truthful on every
transport.

## Why Raw Guest Threading Is Rejected As Canonical Platform Truth

Raw guest threading is not the canonical public model.

Reasons:

- Wasm cannot honestly treat unrestricted guest-owned threading as baseline
  platform truth.
- External-process transport would need a different semantic story again,
  including push channels and cross-process worker ownership.
- Native-only thread semantics would create a false portability promise for SDK
  users.
- Guest-owned threads or task managers would move control of concurrency,
  budgets, shutdown, and telemetry away from the runtime.
- Worker-thread callbacks into guest code would violate the owner-thread
  authoritative apply model and would make reentrancy/ordering transport
  dependent.
- Raw public task/thread APIs would expose implementation mechanics instead of
  stable gameplay semantics.

Host implementations may internally use threads, worker pools, processes, or
other concurrency primitives. That is runtime implementation detail only. It is
not the public semantic contract.

## Design Goals

- One honest model across Wasm, native, and external execution.
- Typed async services rather than generic job submission.
- Explicit submit and explicit completion polling/drain.
- Session-scoped ticket identity and completion ownership.
- Worker computes; owner thread commits.
- Runtime-owned budgets, backpressure, shutdown, and observability.
- No host-pushed completion callbacks as the canonical model.
- No raw public task manager or thread API.
- Clear separation between runtime-only mechanics and intentional SDK/public
  surface.

## Non-Goals

- Defining final SDK function names, Rust traits, FFI structs, or wire layouts.
- Exposing a generic public job queue that accepts arbitrary code or opaque
  work items.
- Defining public guest thread creation, raw wait handles, joins, or task
  schedulers.
- Making guest callbacks reentrant from background workers.
- Solving runtime implementation issues such as worker-pool internals, boot
  integration, or vanilla integration.
- Expanding scope into implementation issues `#24` or `#25`.

## Canonical v1 Model

Canonical v1 async behavior is:

1. The guest submits a request to a typed async service family.
2. The runtime validates the request, session, policy, and capacity.
3. The runtime either rejects submission immediately or accepts it and returns
   a session-scoped ticket.
4. Accepted work executes on host-owned background capacity chosen by the
   runtime.
5. When work reaches a terminal state, the runtime stores a completion record
   for that session.
6. The guest later performs an explicit non-blocking completion poll/drain.
7. Any gameplay-relevant apply/commit step happens on the authoritative owner
   thread after completion observation.

Canonical v1 therefore uses:

- typed service submission
- opaque tickets
- explicit completion polling/drain
- owner-thread commit

Canonical v1 does not use:

- raw guest threads as public truth
- host-pushed completion callbacks
- unsolicited transport messages that reenter the guest
- a generic public task manager

## Typed Async Service Families

Async/background work is defined as a family of typed services, not as a
transport-neutral job runner.

Each async service family must define:

- its semantic purpose
- request type
- terminal result/completion type
- which side(s) may use it
- request/result size and validity rules
- budget and backpressure class
- failure/rejection categories exposed to the guest
- whether the family is optional, gated, or unavailable on some hosts

Async service families must be named and documented at the semantic level.

They must not be defined as:

- arbitrary byte payload execution
- guest-provided function pointers or closures
- raw thread entrypoints
- transport-specific "spawn background task" helpers

The family abstraction is the public unit of async capability. Runtime worker
lanes remain runtime-owned detail.

## Request Submission Semantics

Submission rules:

- Submission is explicit and guest-initiated.
- Submission is scoped to the active runtime session.
- Submission is always to a declared typed service family.
- Submission must be validated before a ticket exists.
- If submission is rejected, no ticket is created.

Immediate submission rejection is the canonical outcome for:

- service family unavailable on the current host/session/side
- invalid request shape or bounds
- policy denial
- backpressure or budget exhaustion
- shutdown or disabled-session state

Acceptance means only that the runtime has taken ownership of the request under
the current session. It does not guarantee:

- immediate execution
- eventual success
- submission-order completion
- uninterrupted execution through shutdown

Submission must remain bounded. The public model must never require unbounded
host queue growth in order to remain correct.

## Ticket Identity, Scope, And Lifecycle

A ticket is the opaque guest handle for one accepted async request.

Ticket rules:

- A ticket is created only by successful submission.
- A ticket is owned by exactly one runtime session.
- A ticket is valid only for the service family that created it.
- A ticket must not be reused across sessions, sides, reloads, or transports.
- A ticket is guest-observable identity, not a capability to mutate runtime
  state directly.

Minimum lifecycle:

1. `accepted`
2. `pending` or `running` under runtime ownership
3. terminal completion record becomes available for drain
4. completion record is drained exactly once
5. ticket is thereafter spent/closed for public purposes

Session end invalidates all outstanding tickets for that session, whether or
not the guest has drained them yet.

Tickets are intentionally opaque. Guests must not infer lane identity, worker
identity, queue position, or transport-specific execution details from them.

## Completion Polling / Drain Semantics

Completion observation is explicit and guest-driven.

Canonical v1 rule:

- the guest learns about completed async work only by calling a non-blocking
  poll/drain operation on the runtime service surface

Polling/drain rules:

- Polling is non-blocking.
- Draining is explicit and consumes completion records.
- A drain may return zero, one, or many completion records.
- A drain may be bounded by host/runtime limits and therefore return only part
  of the currently available completion set.
- Completion records are delivered at most once.
- Completion order is completion availability order, not guaranteed submission
  order.
- Guests must not assume total ordering across different service families.

For canonical v1, "poll" and "drain" are the same semantic family of operation:

- `poll`: non-blocking inspection for available terminal completions
- `drain`: consume available terminal completions, possibly batched

The canonical consumption model is completion drain. A guest that cares about a
specific ticket matches drained completion records against that ticket identity.

Canonical v1 does not require a public blocking wait, join, or future/await
surface.

## Allowed Request / Result Boundary Rules

Async service requests and completion payloads must be honest transport
boundaries.

Allowed boundary properties:

- fully owned data
- transport-stable encoding/representation
- bounded size
- explicit semantic schema
- session-safe identifiers when runtime entities must be referenced

Forbidden boundary content:

- raw pointers or references into guest or host memory
- borrowed slices tied to callback stack lifetime
- closures, function pointers, or executable guest code payloads
- OS thread handles, mutexes, condition variables, or process-local
  synchronization objects
- host object references that bypass contract validation
- direct mutable world/state access tokens

Result payloads may describe computed data or proposals. They must not claim
that authoritative world mutation already happened on a background worker.

## Owner-Thread Authoritative Commit Rule

Background workers compute. The authoritative owner thread commits.

Rules:

- Async workers must not directly apply guest-visible gameplay/world mutation
  as canonical truth.
- Async workers must not reenter guest code with transport-specific completion
  callbacks.
- A drained completion record is an input to later owner-thread logic, not an
  already-applied world mutation.
- Any authoritative apply step remains ordered by owner-thread execution.

This rule preserves the same truthful model across transports and keeps world
state mutation aligned with the runtime's authoritative thread/phase model.

Runtime-owned internal side effects such as caching, queue bookkeeping, and
telemetry may occur off-thread. Those do not change the owner-thread rule for
guest-visible world/application semantics.

## Shutdown Semantics

Shutdown is runtime-owned and session-authoritative.

Rules:

- When a runtime session ends, the runtime must stop accepting new async
  submissions for that session.
- Outstanding accepted work for that session becomes invalid for public
  purposes.
- The runtime may cancel, stop, abandon, or discard queued/running work as
  implementation requires.
- Completed-but-undrained records may be dropped at session end.
- No transport may deliver old completions into a replacement session.
- Session teardown must close the effective completion path for the old
  session.

The guest must treat session end, disable-for-session, unload, hot reload,
reattach, and equivalent runtime reconstruction as terminal boundaries for all
outstanding tickets.

## Cancellation Semantics For v1

Canonical v1 does not define a general public per-ticket cancellation API.

v1 cancellation truth is:

- the guest may stop caring about a ticket locally
- the runtime may still finish, cancel, or discard that work internally
- the guest is not guaranteed a dedicated cancellation acknowledgment
- session end/shutdown is the only guaranteed cancellation boundary

If a future service family adds guest-visible cancellation, it must preserve
the same submit-plus-explicit-completion model and remain transport-parity
honest. v1 does not require that surface.

## Backpressure Semantics

Backpressure is runtime-owned and mandatory.

Rules:

- Submission capacity must be bounded.
- The runtime owns lane widths, queue limits, memory budgets, and per-session
  fairness policy.
- The runtime may reject submission immediately when capacity or policy would
  be exceeded.
- Guests must treat backpressure rejection as an ordinary operational outcome,
  not as a transport fault.
- The public model must not require hidden unbounded buffering to preserve
  semantics.

Drain is also allowed to be bounded:

- the runtime may expose only up to a bounded number of completions per drain
- guests must tolerate draining large completion backlogs over multiple owner
  thread turns

## Observability And Diagnostics Requirements

Async/background execution must remain observable at runtime ownership
boundaries.

The runtime must retain diagnostics sufficient to answer:

- what service family was requested
- whether submission was accepted or rejected
- why a rejection happened at a coarse policy/contract level
- queue depth / in-flight pressure
- execution latency
- completion production and completion drain counts
- drops/abandonment during shutdown or session teardown
- transport/session/mod attribution

Minimum runtime-owned attribution should include where available:

- guest/mod identity
- execution kind
- runtime session identity
- side
- async service family
- ticket identity or ticket count context

Diagnostics are runtime-owned. The public guest surface may expose only coarse,
stable reason categories. Transport-specific internal traces, thread ids, lane
names, or scheduler details are not part of the canonical public model.

## Transport Mapping Rules

All transports must implement the same semantic model:

- typed async service submission
- immediate acceptance or rejection
- opaque session-scoped tickets
- explicit non-blocking completion poll/drain
- owner-thread authoritative commit

### Wasm

- Async submit/drain uses the canonical runtime-service bridge.
- Wasm does not gain host-pushed completion callbacks as canonical truth.
- Wasm does not define raw public guest thread semantics for this model.

### Native

- Async submit/drain uses the canonical native runtime bridge.
- Availability of OS threads in the host process does not change the public
  semantic model.
- Native transport must not redefine the public contract as raw thread/task
  management.

### External

- Async submit/drain uses the canonical explicit IPC service request/response
  path.
- The host must not treat unsolicited completion push messages as canonical
  truth.
- The companion process learns about completed work only when it explicitly
  polls/drains through the runtime service surface.

Transport adapters may optimize internal execution differently. They must not
change the guest-visible model.

## Runtime-Only Versus Intentional SDK/Public Surface

Intentional SDK/public surface includes only:

- typed async service family semantics
- request/result/completion schemas
- explicit submission semantics
- immediate rejection semantics
- opaque session-scoped tickets
- explicit completion polling/drain semantics
- owner-thread authoritative commit rule
- shutdown/backpressure/cancellation guarantees visible to the guest

Runtime-only detail includes:

- worker lane construction
- thread pools, process pools, or executor choice
- queue implementations
- wakeup/parking strategy
- batching/coalescing internals
- internal retry/dedup behavior
- telemetry plumbing detail
- exact scheduling algorithms

There is intentionally no public raw task manager, executor, thread, or worker
API in v1.

## Candidate Future Workloads

Candidate async service families may include workloads such as:

- expensive pathfinding or reachability queries
- spatial analysis / search over large world regions
- procedural candidate generation or scoring
- terrain/worldgen-adjacent background computation
- expensive AI planning or evaluation support work
- bulk derived-data preparation where owner-thread commit still stays explicit

These are candidates only. Each family still requires its own semantic
definition before becoming public contract surface.

## Explicitly Deferred Items

This document intentionally defers:

- generic public job submission
- raw guest thread/task APIs
- host-pushed completion callbacks
- public futures/promises/joins/waits
- progress streaming or partial-result streaming
- guest-visible worker/lane identity
- cross-session ticket persistence or resume
- public priority/QoS controls
- async dependency graphs/chaining semantics
- transport-specific convenience models that change canonical truth
- implementation details for issues `#24` and `#25`
- boot or vanilla integration work

## Canonical v1 Decision Summary

For world-stack runtime-loaded guests, the canonical async public model is:

- typed async services, not raw job submission
- explicit submit and explicit completion poll/drain
- session-scoped opaque tickets and completion records
- worker computes, owner thread commits
- runtime-owned backpressure, shutdown, and observability
- no host-pushed completion callbacks as canonical transport behavior
- no public raw task/thread API
