# Freven SDK

This repository contains the **public Freven SDK** used to build experiences/mods.
The **engine source is private** and is **not** part of this repository.

## Public surface (GitHub-first)

Current SDK crates:
- `freven_api` - stable-ish SDK contracts (pre-1.0)
- `freven_sdk_types` - pure shared SDK types
- `freven_std` - early stdlib helpers (**unstable**; depend only if you accept breakage)

Reference gameplay (vanilla experience) is shipped separately in the `freven-vanilla` repository.

## Depend on the SDK today (git tags)

Use tagged git dependencies until crates.io publishing begins:

```toml
[dependencies]
freven_api = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_api" }
freven_sdk_types = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_sdk_types" }
# Optional / unstable:
# freven_std = { git = "https://github.com/frevenengine/freven-sdk", tag = "v0.1.0", package = "freven_std" }
```

See [docs/sdk/SDK_DISTRIBUTION.md](docs/sdk/SDK_DISTRIBUTION.md) for the rollout plan and release policy.

## Stability notes

- SDK is **pre-1.0**: breaking changes may happen in minor releases.
- Breaking changes will be called out in GitHub tag/release notes.
- Experimental areas will be labeled explicitly in docs/comments.
