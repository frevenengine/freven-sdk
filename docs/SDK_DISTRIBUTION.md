# SDK Distribution Policy (Variant 3)

We follow a **GitHub-first** distribution model for the public Freven SDK.

## What is public-readable now

This repository (`frevenengine/freven-sdk`) is public-readable and contains:
- `freven_api`
- `freven_sdk_types`
- `freven_std` (**unstable**; depend only if you accept breakage)

**Engine source is private and is not part of this repository.**

Reference gameplay (vanilla experience) lives in a separate repository (`frevenengine/freven-vanilla`).

## Publishing plan

Now:
- consume SDK crates via **git tags** from this repository.

Later (planned):
- publish `freven_api` and `freven_sdk_types` to crates.io.
- optionally publish `freven_std` after API hardening.

## How to depend today (Cargo + tag)

```toml
[dependencies]
freven_api = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_api" }
freven_sdk_types = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_sdk_types" }
# Optional / unstable:
# freven_std = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_std" }
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

* Boundary checks green:

  * `python3 scripts/check_layers.py`
  * `python3 scripts/check_boundaries.py --profile phase0 --strict`
* Formatting/lint green:

  * `cargo fmt -- --check`
  * `cargo clippy --workspace --all-targets --all-features -- -D warnings`
* SDK docs reviewed for crate names/path accuracy.
* Tag created and release notes include:

  * stable vs experimental status
  * breaking changes (if any)
  * dependency snippet using `git + tag + package`
