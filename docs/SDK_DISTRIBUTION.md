# SDK Distribution Policy (Variant 3)

We follow a **GitHub-first** distribution model for the public Freven SDK.

## What is public-readable now

This repository (`frevenengine/freven-sdk`) is public-readable and contains:
- `freven_mod_api`
- `freven_guest`
- `freven_guest_sdk`
- `freven_sdk_types`
- `freven_world_api`
- `freven_world_guest`
- `freven_world_guest_sdk`
- `freven_world_sdk_types`

**Engine source is private and is not part of this repository.**

Reference gameplay (vanilla experience) lives in a separate repository (`frevenengine/freven-vanilla`).
First-party gameplay helpers such as break/place payload codecs and humanoid input
now ship from `freven-vanilla`, not from this SDK repository.

## Publishing plan

Now:
- consume SDK crates via **git tags** from this repository.

Later (planned):
- publish `freven_mod_api` and `freven_sdk_types` to crates.io.
- evaluate which explicit `freven_world_*` crates should remain public as the
  world stack hardens.

Naming note:
- `freven_api` has been retired in favor of `freven_mod_api`.
- The new name reflects what the crate actually is: the builtin /
  compile-time mod facade, not the whole public SDK story.

## How to depend today (Cargo + tag)

```toml
[dependencies]
freven_mod_api = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_mod_api" }
freven_guest_sdk = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_guest_sdk" }
freven_sdk_types = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_sdk_types" }
# Explicit world-owned surfaces when you intentionally target the world stack:
# freven_world_guest_sdk = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_world_guest_sdk" }
# freven_world_api = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_world_api" }
````

Guidance:

* pin a `tag` (not a floating branch) for reproducible builds
* use matching tags across Freven crates

## Versioning policy

* SDK is pre-1.0.
* Breaking changes are allowed before 1.0, but must be intentional.
* Every breaking change must be called out in GitHub release notes.
* Patch releases should remain backward compatible.

## Lightweight release checklist

* Formatting/lint green:

  * `cargo fmt -- --check`
  * `cargo clippy --workspace --all-targets --all-features -- -D warnings`
* Tests green:

  * `cargo test --workspace --all-features`
* Additional boundary/layer checks exist in the private engine repo, but are not part of the public SDK repo checklist.
* SDK docs reviewed for crate names/path accuracy.
* Tag created and release notes include:

  * stable vs experimental status
  * breaking changes (if any)
  * dependency snippet using `git + tag + package`
