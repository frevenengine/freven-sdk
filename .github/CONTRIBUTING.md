# Contributing to Freven SDK

Thanks for your interest in contributing.

## Ground rules

- Keep changes scoped and intentional.
- Prefer small, reviewable pull requests.
- Discuss large architectural changes before implementing them.
- Follow existing crate/layer boundaries and public contract direction.

## What contributions are accepted

- Bug fixes
- Documentation improvements
- Small, well-scoped SDK ergonomics improvements
- Contract / ABI / docs clarifications
- Tests and validation improvements

## What contributions are NOT accepted

- Large refactors without prior discussion
- Broad "rewrite everything" pull requests without a clear architectural reason
- Changes that collapse the neutral SDK roots and explicit world-owned SDK surfaces back into one mixed surface

## How to contribute

- Contributions are accepted via Pull Requests to the official upstream repository.
- Keep PRs small and focused.
- Include tests or a clear validation story when behavior changes.
- Update docs/examples when public SDK behavior changes.

## Licensing

By intentionally submitting a contribution to this repository, you agree that
your contribution is provided under the same license as this repository,
Apache License 2.0, unless explicitly stated otherwise.

## Local checks before opening a PR

Run:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`

## SDK tags / versioning policy

- If a PR changes public SDK contracts or public author-facing behavior, it may require a version bump and a new git tag.
- Downstream repositories should prefer tagged SDK versions over floating revisions.
