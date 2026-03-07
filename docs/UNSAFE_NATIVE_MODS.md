# Unsafe Native Mods

Native guest execution is opt-in and disabled by default.

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

## Canonical model

Disk-loaded native guests use this semantic model in `mod.toml`:

- `artifact = "native_library"`
- `execution = "native_guest"`
- `trust = "trusted"`
- `policy = "unsafe_native"`

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

- `freven_guest_alloc(size: u32) -> u32`
- `freven_guest_dealloc(ptr: u32, len: u32)`
- `freven_guest_negotiate(payload_ptr: u32, payload_len: u32) -> u64`
- `freven_guest_handle_action(payload_ptr: u32, payload_len: u32) -> u64`
- optional lifecycle exports when declared in `GuestDescription`:
  - `freven_guest_on_start_client`
  - `freven_guest_on_start_server`
  - `freven_guest_on_tick_client`
  - `freven_guest_on_tick_server`

`freven_guest_negotiate` returns postcard bytes for `NegotiationResponse`
packed as `(ptr,len)`.
`freven_guest_handle_action` returns postcard bytes for `ActionResult` packed as
`(ptr,len)`.
Lifecycle exports return postcard bytes for `LifecycleAck` packed as `(ptr,len)`.
Action input bytes passed to `freven_guest_handle_action` are postcard
`ActionInput`, which is the only authority for action binding and runtime
action context.

Packed format matches WASM ABI v1 exactly:

- `((ptr as u64) << 32) | (len as u64)`

See [NATIVE_MOD_ABI_v1.md](./NATIVE_MOD_ABI_v1.md) for exact details.

## Policy notes

- Native mods are never loaded unless explicit opt-in is enabled.
- If an explicitly required disk `mod.toml` resolves to `policy = "unsafe_native"` while opt-in is disabled, resolution fails with an actionable error that includes the manifest path and enable flag/env.
- If an explicitly required disk `mod.toml` resolves to `policy = "external_process"` while external policy is disabled, resolution fails with an actionable error that includes the manifest path and enable flag/env.
- Builtin registrations are no longer modeled as native/external candidates.
- Native mods are local-only and not treated as server-downloadable artifacts.
