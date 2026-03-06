# Unsafe Native Mods

Native mods (`kind = "native"`) are opt-in and disabled by default.

The canonical public guest contract is `freven_guest`. Native loading remains a
separate unsafe transport path and is not the primary guest contract surface.

## Enable

Use one of:

- CLI: `--unsafe-native-mods`
- Environment: `FREVEN_UNSAFE_NATIVE_MODS=1`

Supported entry points:

- `freven_boot run --unsafe-native-mods ...`
- `freven_server --unsafe-native-mods ...`
- `freven_client --unsafe-native-mods ...`

When enabled, Freven logs a loud warning because native libraries run with full process privileges.

## On-disk location

Native library files are loaded from:

- `<instance>/unsafe_native_mods/<modid>/<entry>`

Example:

- mod id: `acme.test.native`
- `entry` in `mod.toml`: `bin/libacme_native.so`
- runtime load path: `<instance>/unsafe_native_mods/acme.test.native/bin/libacme_native.so`

`entry` must be a relative path and must not contain `..`.
Absolute paths, root/prefix components, and parent traversal are rejected during mod resolution.

## ABI boundary

Native mods use the unified ABI surface shared with WASM/runtime contracts:

- `freven_alloc(size: u32) -> u32`
- `freven_dealloc(ptr: u32, len: u32)`
- `freven_init() -> u64`
- `freven_handle_action(kind: u32, payload_ptr: u32, payload_len: u32) -> u64`

`freven_init` returns postcard bytes for `ModManifestV1` packed as `(ptr,len)`.
`freven_handle_action` returns postcard bytes for `ActionResultV1` packed as `(ptr,len)`.
Action input bytes passed to `freven_handle_action` are postcard `ActionInputV1`, which is the
only authority for `player_id` and `at_input_seq`.

Packed format matches WASM ABI v1 exactly:

- `((ptr as u64) << 32) | (len as u64)`

See [NATIVE_MOD_ABI_v1.md](./NATIVE_MOD_ABI_v1.md) for exact details.

## Policy notes

- Native mods are never loaded unless explicit opt-in is enabled.
- If an explicitly required disk `mod.toml` resolves to `kind = "native"` while opt-in is disabled, resolution fails with an actionable error that includes the manifest path and enable flag/env.
- If an explicitly required disk `mod.toml` resolves to `kind = "external"` while external policy is disabled, resolution fails with an actionable error that includes the manifest path and enable flag/env.
- Builtin native/external candidates may be skipped when the corresponding policy is disabled.
- Native mods are local-only and not treated as server-downloadable artifacts.
