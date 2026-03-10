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

Native mods use the same semantic guest contract as Wasm, but not the same
memory ABI. Native uses explicit in-process FFI structs:

- `freven_guest_alloc(size: usize) -> *mut u8`
- `freven_guest_dealloc(buffer: NativeGuestBuffer)`
- `freven_guest_negotiate(input: NativeGuestInput) -> NativeGuestBuffer`
- `freven_guest_handle_action(input: NativeGuestInput) -> NativeGuestBuffer`
- optional lifecycle exports when declared in `GuestDescription`:
  - `freven_guest_on_start_client`
  - `freven_guest_on_start_server`
  - `freven_guest_on_tick_client`
  - `freven_guest_on_tick_server`

`NativeGuestInput` is `#[repr(C)] { ptr: *const u8, len: usize }`.
`NativeGuestBuffer` is `#[repr(C)] { ptr: *mut u8, len: usize }`.

Zero-length native inputs and outputs are canonical only as `ptr = null` with `len = 0`.
Non-null zero-length buffers are invalid.

`freven_guest_negotiate` returns postcard bytes for `NegotiationResponse`.
`freven_guest_handle_action` returns postcard bytes for `ActionResult`.
Lifecycle exports return postcard bytes for `LifecycleResult`.
Action input bytes passed to `freven_guest_handle_action` are postcard
`ActionInput`, which is the only authority for action binding and runtime
action context.

Native guests may also expose `freven_guest_set_native_runtime_bridge(...)` to
receive canonical runtime service requests during lifecycle, action, or message
callbacks.

For non-empty input, the host allocates guest-owned input buffers with
`freven_guest_alloc`, passes them by `NativeGuestInput`, and releases them with
`freven_guest_dealloc`. Empty input is passed canonically as `ptr = null` with
`len = 0`.

Returned `NativeGuestBuffer` values are copied and then released with
`freven_guest_dealloc`. Zero-length output is canonical only as `ptr = null`
with `len = 0`.

See [NATIVE_MOD_ABI_v1.md](./NATIVE_MOD_ABI_v1.md) for exact details.

## Policy notes

- Native mods are never loaded unless explicit opt-in is enabled.
- If an explicitly required disk `mod.toml` resolves to `policy = "unsafe_native"` while opt-in is disabled, resolution fails with an actionable error that includes the manifest path and enable flag/env.
- If an explicitly required disk `mod.toml` resolves to `policy = "external_process"` while external policy is disabled, resolution fails with an actionable error that includes the manifest path and enable flag/env.
- Builtin registrations are no longer modeled as native/external candidates.
- Native mods are local-only and not treated as server-downloadable artifacts.
