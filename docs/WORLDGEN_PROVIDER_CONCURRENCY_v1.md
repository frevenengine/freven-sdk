# Worldgen Provider Concurrency Contract v1

This document defines the canonical SDK-owned concurrency contract for
`freven_volumetric_api::WorldGenProvider` and `WorldGenFactory`.

It is the semantic truth that builtin mods, runtime hosts, and future SDK/API
work must preserve before any worldgen-provider parallelism is widened.

This is a docs-first contract definition. It does not enable parallel worldgen,
define runtime APIs, or change current behavior.

## Scope

- Semantic contract only.
- Owned by `freven-sdk`.
- Applies to provider/session execution for volumetric worldgen providers.
- Separate from transport-specific worker mechanics and separate from generic
  async service APIs.

`WORLD_ASYNC_SERVICE_MODEL_v1.md` defines the generic guest-facing async model.
This document defines the narrower concurrency contract for worldgen provider
execution itself.

## Provider Session Definition

A provider session is the host-owned logical execution context for one active
worldgen provider identity within one active runtime/world binding.

For Wasm guests, workload-scoped `worldgen_*` capability requests apply only
when the runtime actually hosts that worldgen provider. A both-side mod may
declare a worldgen provider and `worldgen_*` requests while still attaching on
the client side; that client attachment does not create a client-side worldgen
provider session.

In the current model, a host may realize that session as:
- one builtin provider instance bound to one active world/session, or
- one hosted runtime-loaded guest worldgen session wrapped by transport/runtime-specific plumbing.

Regardless of transport, the semantic rule is the same: `serial_session`
forbids overlapping `generate` calls on that provider session.

Transport-specific internal locking or serialization does not widen the public
contract into shared-instance parallel execution.

## Canonical Modes

The canonical mode names are:

- `serial_session`
- `parallel_isolated_job`

No other mode is reserved as public semantic truth in v1.

### Current Canonical Mode: `serial_session`

`serial_session` is the canonical default and the only active mode in this
phase.

Rules:

- One provider instance is owned by one world/session.
- The host invokes `generate` sequentially for that provider/session.
- Calls to `generate` on the same provider/session must not overlap.
- Current SDK/runtime behavior must continue to match this mode unless a later
  contract revision explicitly widens it.

`serial_session` is therefore the only truthful interpretation of the current
SDK provider traits and docs.

### Reserved Future Mode: `parallel_isolated_job`

`parallel_isolated_job` is the only future widening path worth reserving.

It is reserved only. It is not enabled by this document.

If a future contract revision activates `parallel_isolated_job`, it must define
explicitly:

- what constitutes one job
- how job input/state isolation is established
- how outputs are returned to the owner thread
- how stale results are detected and discarded
- how shutdown/cancellation behaves
- how faults are surfaced and how the host stops further use

The semantic boundary is:

- parallelism may exist across isolated jobs
- isolation must prevent correctness from depending on shared mutable provider
  state across those jobs
- owner-thread commit remains authoritative

This reserved mode does not authorize a host to reinterpret the current API as
"parallel if it seems to work in Rust."

### Unsupported / Forbidden Mode: Shared-Instance Parallel Execution

Shared-instance concurrent `generate` on one provider/session is unsupported
and non-canonical.

That includes:

- overlapping `generate` calls on the same provider instance
- overlapping `generate` calls that share one mutable session-owned provider
  state object
- inferring permission from internal locks, atomics, or other implementation
  details
- inferring permission from `Send + Sync`

If a host wants future parallelism, it must use an explicitly defined
`parallel_isolated_job` contract revision instead of shared-instance concurrent
callbacks.

## `Send + Sync` Interpretation

`WorldGenProvider: Send + Sync` and `WorldGenFactory: Send + Sync` are
memory-safety / host-integration bounds only.

They do not define the gameplay/runtime correctness contract.

Specifically:

- they do not grant permission for shared-instance concurrent `generate`
- they do not imply thread-safe semantics are sufficient for deterministic
  worldgen correctness
- they do not widen the canonical execution model beyond `serial_session`

Rust auto traits answer "may this value cross or be referenced across thread
boundaries safely enough for Rust's memory model?" They do not answer "is
shared-instance concurrent worldgen a supported semantic contract?"

This document owns the latter answer, and in v1 that answer is "no."

## Determinism And Correctness Requirements

Worldgen correctness must not depend on:

- worker count
- CPU count
- scheduling/interleaving order
- thread identity
- wall clock / current time
- hidden shared mutable state

Equivalent semantic inputs must produce equivalent worldgen outputs regardless
of whether execution is current `serial_session` or any future permitted
`parallel_isolated_job` widening.

Performance-only caches are allowed only if they are observationally inert:
they may change cost, but they must not change generated output.

## Owner-Thread Commit Rule

Worker computes, owner thread commits remains true.

Rules:

- provider execution may be hosted however the runtime chooses internally
- computed terrain writes are not authoritative merely because they were
  produced
- any gameplay/world commit step remains host-owned and owner-thread
  authoritative
- background workers must not become an alternate public commit authority

This keeps worldgen execution compatible with the broader async model defined in
`WORLD_ASYNC_SERVICE_MODEL_v1.md`.

## Session Ownership, Shutdown, Stale, And Fault Semantics

Provider execution remains host-owned and session-scoped.

Rules:

- a provider instance belongs to one active world/session
- session end, unload, reload, or host shutdown makes further use of that
  provider/session stale
- stale or shutdown-affected work must not be committed as authoritative world
  state
- the host may stop, cancel, abandon, or ignore in-flight work when session
  ownership ends
- fault handling must remain explicit; stale, shutdown, and fault outcomes must
  not be silently treated as successful generation
- if a provider/session is faulted or disabled, runtime-owned wrappers must
  stop further invocation for that session

Any future `parallel_isolated_job` activation must preserve these explicit
host-owned lifecycle semantics. It must not introduce detached guest-owned
worker lifetimes that outlive session ownership.

## v1 Decision Summary

For SDK-owned worldgen provider execution:

- current canonical mode is `serial_session`
- reserved future mode is `parallel_isolated_job`
- shared-instance concurrent `generate` is unsupported / forbidden
- `Send + Sync` are memory-safety bounds only, not correctness permission
- determinism must not depend on worker count, CPU count, schedule order,
  thread identity, wall clock, or hidden shared mutable state
- worker computes, owner thread commits remains true
- shutdown, stale, and fault semantics stay explicit and host-owned
