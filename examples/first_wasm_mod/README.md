# First Wasm mod example

This is a minimal Freven runtime-loaded Wasm world mod.

It demonstrates the normal public authoring path:

- `freven_world_guest_sdk::wasm_guest!`
- server lifecycle logging
- a custom block registration
- a simple worldgen provider
- a bootstrap spawn hint
- a current `schema = 3` `mod.toml`

## Build

From the `freven-sdk` repository root:

```bash
rustup target add wasm32-unknown-unknown
cargo +stable build \
  --manifest-path examples/first_wasm_mod/Cargo.toml \
  --release \
  --target wasm32-unknown-unknown
```

The artifact is written to:

```text
examples/first_wasm_mod/target/wasm32-unknown-unknown/release/freven_first_wasm_mod.wasm
```

Install it into a DevKit instance as:

```text
<instance>/mods/example.first_wasm/mod.wasm
<instance>/mods/example.first_wasm/mod.toml
```

Then reference it from an `experience.stack.toml` layer:

```toml
schema = 1
id = "example.first_wasm.stack"
version = "0.1.0"
title = "Vanilla + First Wasm Mod"
base = "freven.vanilla"

[[layers]]
id = "example.first_wasm.layer"
version = "0.1.0"
title = "First Wasm Worldgen"

[layers.defaults]
worldgen = "example.first_wasm:flat"

[[layers.mods]]
id = "example.first_wasm"
version = "^0.1"
```

Use the DevKit provider diagnostics before launch:

```bash
./freven_boot providers explain --instance <instance> --experience example.first_wasm.stack
./freven_boot providers check --instance <instance> --experience example.first_wasm.stack
```

See the DevKit `docs/FIRST_WASM_MOD.md` guide for the full from-scratch workflow.
