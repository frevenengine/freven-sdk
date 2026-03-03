# Contributing to Freven SDK

This repository is **readable source** under the Freven Readable Source License (FRSL).
**Redistribution, mirroring, and public forks are not permitted by the license.**

## What contributions are accepted
- Bug fixes
- Documentation improvements
- Small, well-scoped improvements to SDK ergonomics
- ABI/docs clarifications

## What contributions are NOT accepted
- Requests to open-source the engine or change the licensing model
- Large refactors without prior discussion
- "Rewrite the world" PRs that touch many files without a clear reason

## How to contribute
- Contributions are accepted **only via Pull Requests** to the official upstream repository.
- Keep PRs small and focused (one change = one PR).
- Include tests or a clear validation story.

## By contributing
By submitting a PR, you confirm you have the right to contribute the code and you agree that
your contribution may be used, modified, and relicensed by Freven as part of the project
(as described in the repository LICENSE).

## Local checks before opening a PR
Run:
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`

## SDK tags / versioning policy
- If a PR changes the public contracts in `freven_api`, `freven_sdk_types`, or `freven_std`,
  it requires a **version bump** and a **new git tag** (e.g. `v0.1.1`).
- Downstream repos (like `freven-vanilla`) must update dependencies **only via tags**.
