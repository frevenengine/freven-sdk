# Mod Config v1

This document defines the public Freven mod configuration authoring path.

For the broader manifest / config schema / active config / content data /
assets / generated cache / save-state boundary model, see
[PACKAGE_BOUNDARIES.md](PACKAGE_BOUNDARIES.md).

The core rule is simple:

- `mod.toml` is manifest, execution, trust, surface, entrypoint, capability, and
  schema-reference metadata.
- `config.schema.toml` declares the mod's supported settings and defaults.
- active runtime values are authored by the selected experience or experience
  stack.
- guests receive only the final resolved per-mod configuration through
  `StartInput.config`.

`mod.toml [config]` is intentionally not a supported runtime config path. Hosts,
tools, and users should not treat it as active gameplay configuration.

## Files and ownership

A mod may reference a schema file from `mod.toml`:

```toml
schema = 3
id = "example.hello"
version = "0.1.0"
artifact = "wasm_module"
execution = "wasm_guest"
trust = "sandboxed"
policy = "safe_guest"
surfaces = "server"
entry = "mod.wasm"
config_schema = "config.schema.toml"
```

`config_schema` is a safe relative path next to the mod manifest. It must not be
absolute and must not escape with `..`.

The schema file declares settings, defaults, validation constraints, scope,
reload policy, and authority:

```toml
schema = 1

[[settings]]
key = "enabled"
type = "bool"
default = true
scope = "server_world"
reload = "runtime"
authority = "server"

[[settings]]
key = "max_mutations_per_tick"
type = "int"
default = 128
min = 1
max = 4096
scope = "server_world"
reload = "world_restart"
authority = "server"

[[settings]]
key = "gravity"
type = "float"
default = 9.8
min = 0.0
max = 20.0
scope = "server_world"
reload = "world_restart"
authority = "server"

[[settings]]
key = "difficulty"
type = "enum"
default = "normal"
allowed_values = ["easy", "normal", "hard"]
scope = "server_world"
reload = "world_restart"
authority = "server"
```

Supported setting types are `bool`, `int`, `float`, `string`, and `enum`.
Supported scopes are `startup`, `server_world`, and `client_user`.
Supported reload policies are `restart`, `world_restart`, `reconnect`, and
`runtime`.
Supported authorities are `server`, `client`, and `user`.

The schema is a declaration and default source. It is not the user's active
configuration file.

## Active config in an experience

An experience authors active values with a table per mod id:

```toml
[[mods]]
id = "example.hello"
version = "^0.1"

[config."example.hello"]
enabled = true
max_mutations_per_tick = 256
gravity = 8.5
difficulty = "hard"
```

For mods with a schema, the resolver starts from schema defaults and applies the
authored values. Unknown keys, invalid value types, invalid enum values, and
out-of-bounds numeric values are rejected before runtime start.

For mods without a schema, the authored per-mod table is preserved as-is. This
keeps early/custom mods usable while schema-backed mods get validation and
tooling support.

## Active config in an experience stack

A stack layer can override the base experience config:

```toml
[[layers]]
id = "example.stack.layer"
title = "Server Balance"

[layers.config."example.hello"]
max_mutations_per_tick = 512
difficulty = "normal"
```

Stack layer config deep-merges over the base experience config. If both sides
contain tables, nested keys are merged recursively. Otherwise the layer value
replaces the base value.

Current public authored layers are experience config and stack-layer config.
Future world or instance config layers should compose into the same resolved
effective per-mod config model rather than adding a separate guest-facing side
channel.

## Guest access

Guests do not read `mod.toml`, `config.schema.toml`, the experience file, or the
stack file directly. The host resolves the final effective config and sends it
in `StartInput.config`.

With `freven_world_guest_sdk`:

```rust
use serde::Deserialize;
use freven_world_guest_sdk::{LifecycleResult, StartInput, StartInputExt};

#[derive(Debug, Deserialize)]
struct Config {
    enabled: bool,
    max_mutations_per_tick: u32,
    gravity: f32,
    difficulty: String,
}

fn start_server(input: &StartInput) -> LifecycleResult {
    let config: Config = input
        .config_typed()
        .expect("resolved config should match the declared schema");

    if config.enabled {
        freven_world_guest_sdk::log_info!(
            "difficulty={} gravity={}",
            config.difficulty,
            config.gravity
        );
    }

    LifecycleResult::default()
}
```

`StartInput.config` is currently serialized as TOML text inside
`ModConfigDocument`. The transport shape is an implementation detail of the
guest contract; semantically, it is the final resolved per-mod config.

## Verification

DevKit users can inspect the resolved config before starting a runtime:

```bash
freven_boot config explain --instance <instance> --experience <experience_id>
freven_boot config explain --instance <instance> --experience <experience_id> --mod example.hello
```

This command resolves the selected experience/stack and prints the same effective
per-mod config that guests receive at start.

## Reload and compatibility notes

`reload` is public metadata for tooling and host policy. A setting marked
`runtime` may be safe to apply while running, while `world_restart`, `reconnect`,
or `restart` indicates a stronger lifecycle boundary. Existing worlds or already
running servers may still need a restart or recreation depending on what the mod
does with the setting.

Do not add ad hoc config channels for a specific transport. Wasm, native,
external-process, and builtin/compile-time authoring should all converge on the
same resolved effective config semantics.
