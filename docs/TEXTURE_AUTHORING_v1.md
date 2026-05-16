# Texture Authoring v1

This document defines the Freven rc10 texture authoring, sampling, mipmap, alpha,
and validation policy.

It builds on:

- [ARCHITECTURE.md](ARCHITECTURE.md): engine / SDK / experience / mod /
  content-pack / standalone-product ownership;
- [PACKAGE_BOUNDARIES.md](PACKAGE_BOUNDARIES.md): content data, assets,
  generated cache, and save/world state ownership;
- [VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md): stable visual asset
  identity and dependency graph;
- [MATERIAL_DEFINITIONS_v1.md](MATERIAL_DEFINITIONS_v1.md): material texture
  references and sampling profile references;
- [LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md): texture
  override policy and diagnostics.

## Goals

- Define texture source files as authored assets, not material definitions and
  not generated cache.
- Give mod authors clear texture size, format, alpha, sampling, and mipmap rules.
- Define conservative voxel texture profiles that work for blocks, items,
  materials, and future renderer pipelines.
- Keep atlas coordinates, texture-array layers, renderer slots, GPU samplers,
  Bevy handles, wgpu handles, and cache paths out of authored texture data.
- Make validation diagnostics predictable and useful for DevKit users.
- Leave atlas or texture-array packing to the dedicated
  [TEXTURE_BACKEND_PIPELINE_v1.md](TEXTURE_BACKEND_PIPELINE_v1.md) document.

## Non-goals

This document does not define:

- atlas packing;
- texture-array layer assignment;
- generated cache directory layout;
- renderer bind groups or GPU resource lifetime;
- Vanilla's actual texture library;
- image editor recommendations;
- arbitrary shader/plugin texture inputs;
- compressed runtime transcode/cache format.

Those belong to follow-up engine, SDK, Vanilla, and DevKit work.

## Core rule

Authors reference texture assets through stable visual asset keys.

Author-facing texture identity:

~~~text
example.gems:textures/block/ruby_ore
freven.vanilla:textures/block/stone
example.ui:textures/icons/warning
~~~

Author-facing package-local source file:

~~~text
assets/textures/block/ruby_ore.png
assets/textures/icons/warning.png
~~~

Host/backend output:

~~~text
atlas coordinate
texture-array layer
GPU texture/view/sampler
Bevy Handle<Image>
compiled/transcoded texture cache
generated load-plan node
~~~

Only the first two are author-facing. Backend output is not stable SDK
vocabulary.

## Ownership and location

Texture source files live in `assets/`, normally under:

~~~text
assets/textures/
~~~

Material definitions live in `content/materials/` and reference textures by key.
Generated atlases, texture arrays, transcoded images, fingerprints, and load
plans live in generated cache.

Example package layout:

~~~text
mods/example.gems/
  mod.toml
  content/
    materials/
      block/
        ruby_ore.toml
  assets/
    textures/
      block/
        ruby_ore.png
        ruby_ore_normal.png
        ruby_ore_mr.png
~~~

Rules:

- texture files are authored assets;
- a texture file is not a material definition;
- a generated atlas is not authored source;
- generated cache may be deleted and rebuilt;
- save/world state must not store texture source files.

## Stable texture keys

Texture keys use the shared visual asset key shape:

~~~text
namespace:path
~~~

Examples:

~~~text
freven.vanilla:textures/block/stone
freven.vanilla:textures/block/grass_side
example.gems:textures/block/ruby_ore
example.gems:textures/item/ruby
example.space:textures/hull/panel_albedo
example.space:textures/hull/panel_normal
example.space:textures/hull/panel_mr
example.ui:textures/icons/warning
~~~

Rules:

- the namespace owns the texture identity;
- the path identifies the texture inside that namespace;
- keys should be stable across compatible releases;
- keys must not encode file system install paths, atlas coordinates,
  texture-array layers, renderer slots, runtime ids, GPU handles, or cache paths;
- one selected stack must resolve each effective texture key to one texture asset
  declaration;
- replacing texture bytes is an asset override decision, not a material content
  patch by itself.

## Source format policy

### Required rc10 baseline

The required rc10 source format is:

| Format | Role |
| --- | --- |
| PNG | Required baseline source format for authored textures. |

PNG is the required authoring baseline because it is portable, easy to inspect,
easy to hash, and widely supported by asset tools.

### Future or optional formats

Future tooling may support:

| Format | Role |
| --- | --- |
| KTX2 / Basis Universal | Optional compressed or transcodable texture source. |
| WebP | Optional source format if product/tooling policy allows it. |
| EXR / HDR | Optional high-dynamic-range source for advanced effects. |

Rules:

- optional formats must be declared by tooling/runtime capability;
- unsupported formats are diagnostics;
- accepting a compressed source format must not expose GPU upload details as
  author-facing ids;
- generated transcodes are cache, not authored source of truth.

## Texture profiles

Texture validation depends on a texture profile.

A profile is author-facing policy. It is not a GPU sampler, renderer object, or
atlas slot.

Canonical v1 profiles:

| Profile | Typical use |
| --- | --- |
| `voxel_block` | Block face textures, terrain-style voxel textures. |
| `voxel_item` | Item icons and inventory-style voxel assets. |
| `model_surface` | Entity, prop, equipment, or imported model textures. |
| `ui` | Interface icons and UI surfaces. |
| `effect_sprite` | Particles, decals, billboards, and simple effects. |

When no profile is declared, tooling should infer one from context:

| Context | Default profile |
| --- | --- |
| material referenced by block visual | `voxel_block` |
| material referenced by item visual | `voxel_item` |
| material referenced by entity/model visual | `model_surface` |
| UI visual | `ui` |
| particle/effect visual | `effect_sprite` |

DevKit diagnostics should show both the inferred profile and why it was chosen.

## Size policy

### `voxel_block`

`voxel_block` textures are the strictest profile.

Accepted v1 rules:

- width and height must be equal;
- width and height must be powers of two;
- recommended sizes are `16x16`, `32x32`, and `64x64`;
- `128x128` and `256x256` are allowed but may produce performance warnings;
- larger sizes require explicit product/tooling policy;
- zero-sized images are invalid;
- non-square images are invalid;
- non-power-of-two images are invalid.

Examples:

| Size | Result |
| --- | --- |
| `16x16` | accepted |
| `32x32` | accepted |
| `64x64` | accepted |
| `128x128` | accepted with possible warning |
| `256x256` | accepted with possible warning |
| `16x32` | rejected for `voxel_block` |
| `48x48` | rejected for `voxel_block` |
| `0x0` | rejected |
| `4096x4096` | rejected unless product policy explicitly allows it |

### `voxel_item`

`voxel_item` textures are also normally square and power-of-two.

Accepted v1 rules:

- square power-of-two is recommended;
- `16x16`, `32x32`, `64x64`, `128x128`, and `256x256` are accepted;
- non-square item textures require explicit profile override or future UI/model
  visual policy;
- very large item textures should warn or fail by product policy.

### `model_surface`

`model_surface` textures may be less restrictive.

Accepted v1 rules:

- power-of-two is recommended;
- square and rectangular textures are allowed;
- dimensions must be positive;
- maximum size is product/tooling policy;
- non-power-of-two may be accepted with a warning if the selected renderer and
  platform policy support it;
- mipmap policy should be explicit.

### `ui`

`ui` textures may be rectangular.

Accepted v1 rules:

- rectangular textures are allowed;
- power-of-two is not required by default;
- mipmaps are normally disabled;
- filtering depends on the selected UI sampling profile;
- very large UI textures should warn or fail by product policy.

### `effect_sprite`

`effect_sprite` textures may be square or rectangular.

Accepted v1 rules:

- dimensions must be positive;
- power-of-two is recommended when mipmaps are enabled;
- animation sheets must declare frame layout in a future effect/sprite schema;
- atlas packing remains generated-cache behavior.

## Sampling profiles

Sampling profile names are author-facing policy values.

Canonical v1 profiles:

| Profile | Filter intent | Mip intent | Typical use |
| --- | --- | --- | --- |
| `voxel_nearest` | nearest | no mipmaps | crisp pixel-art block textures |
| `voxel_nearest_mip` | nearest | generated or supplied mips | pixel-art blocks with distance stability |
| `voxel_linear_mip` | linear | generated or supplied mips | smoother high-res voxel packs |
| `model_linear_mip` | linear | generated or supplied mips | model/entity surfaces |
| `ui_nearest` | nearest | no mipmaps | pixel UI |
| `ui_linear` | linear | no mipmaps | smooth UI |
| `effect_linear_mip` | linear | generated or supplied mips | sprites, decals, particles |

Rules:

- profiles are stable names, not GPU sampler handles;
- renderer-specific sampler objects are internal;
- unsupported profiles are diagnostics;
- profile selection should be deterministic;
- profile defaults may depend on texture profile and material context.

Default mapping:

| Texture profile | Default sampling |
| --- | --- |
| `voxel_block` | `voxel_nearest` |
| `voxel_item` | `voxel_nearest` |
| `model_surface` | `model_linear_mip` |
| `ui` | `ui_nearest` |
| `effect_sprite` | `effect_linear_mip` |

Products may override defaults through explicit policy, but the effective policy
must be inspectable by DevKit.

## Mipmap policy

Mipmap behavior is controlled by sampling profile and product/tooling policy.

Canonical v1 mip policies:

| Policy | Meaning |
| --- | --- |
| `none` | Do not use mipmaps. |
| `generate` | Tooling/host may generate mipmaps from source. |
| `provided` | Source package must provide explicit mip levels. |
| `optional` | Use provided mips if present, otherwise generate or omit by profile policy. |

Rules:

- `voxel_nearest` defaults to `none`;
- `voxel_nearest_mip`, `voxel_linear_mip`, `model_linear_mip`, and
  `effect_linear_mip` default to `generate`;
- `ui_nearest` and `ui_linear` default to `none`;
- if mipmaps are required but cannot be generated or loaded, validation should
  fail or downgrade with an explicit diagnostic according to strictness policy;
- generated mipmaps are cache, not authored source truth;
- atlas/array mip layout belongs to the atlas/array pipeline.

## Alpha policy

Texture alpha must match the material alpha intent.

### Opaque materials

For `alpha_mode = "opaque"`:

- alpha may be absent;
- alpha may be present but ignored;
- non-fully-opaque alpha should warn in strict validation because it may indicate
  the wrong material alpha mode.

### Cutout materials

For `alpha_mode = "cutout"`:

- the base color texture should have an alpha channel;
- alpha cutoff comes from material policy, normally `alpha_cutoff = 0.5`;
- if no alpha channel exists, validation should warn or fail depending on
  strictness;
- cutout materials should normally use `render_layer = "cutout"`.

### Blend materials

For `alpha_mode = "blend"`:

- the base color texture should have an alpha channel or the material should
  define a transparent `base_color_rgba`;
- blended materials should normally use `render_layer = "transparent"`;
- transparent sorting and renderer-specific behavior are not authored texture
  identity.

## Color-space policy

Texture fields have semantic color-space expectations.

| Material field | Texture kind | Expected color space |
| --- | --- | --- |
| `base_color_texture` | color | sRGB |
| `emissive_texture` | color | sRGB |
| `normal_texture` | data | linear/data |
| `metallic_roughness_texture` | data | linear/data |
| `occlusion_texture` | data | linear/data |
| future masks | data | linear/data |

Rules:

- color textures should be decoded as color data;
- data textures must not be color-corrected as sRGB;
- color-space metadata mismatch is a diagnostic;
- exact renderer upload format is backend-internal;
- product/tooling policy may choose concrete GPU formats.

## Channel conventions

Material v1 references texture keys but does not fully freeze every packed map
convention. The minimum v1 convention is:

| Texture field | Channel expectation |
| --- | --- |
| `base_color_texture` | RGBA color. |
| `normal_texture` | tangent-space normal data. |
| `emissive_texture` | RGB emissive color, optional alpha ignored unless future policy defines it. |
| `occlusion_texture` | single-channel or packed data by future policy. |
| `metallic_roughness_texture` | packed data, exact channel convention owned by material/texture policy and DevKit validation. |

Until engine renderer support is complete, tools should preserve these references
as authored intent and produce clear diagnostics for unsupported combinations.

## Validation strictness

Tooling should support at least two modes:

| Mode | Meaning |
| --- | --- |
| `authoring` | Helpful warnings, intended for iteration. |
| `release` | Strict validation, intended for packaging and CI. |

In `authoring` mode, oversized textures, alpha mismatches, non-power-of-two model
textures, and unsupported optional metadata may be warnings.

In `release` mode, invalid dimensions, unsupported formats, missing files,
missing required alpha, and unknown profiles should fail.

## Diagnostics

Validators should report at least:

| Diagnostic | Should include |
| --- | --- |
| Missing texture file | texture key, expected package-relative path, owner package/layer |
| Unsupported file format | texture key, file path, accepted formats |
| Decode failure | texture key, file path, decoder error |
| Invalid dimensions | texture key, actual width/height, active profile |
| Non-square voxel texture | texture key, actual width/height, required profile |
| Non-power-of-two voxel texture | texture key, actual width/height |
| Oversized texture | texture key, actual size, policy max |
| Missing alpha for cutout/blend | material key, texture key, alpha mode |
| Unexpected alpha on opaque | material key, texture key |
| Unknown sampling profile | texture/material key, profile, accepted profiles |
| Unsupported mip policy | texture key, profile, policy |
| Color-space mismatch | texture key, material field, expected color space |
| Renderer-internal id used | source file, field, forbidden value |
| Atlas/cache path used as source | source file, field, forbidden value |

Diagnostics should be stable enough for DevKit output, CI, and docs.

## Relationship to material definitions

Material files reference texture keys:

~~~toml
schema = 1
key = "example.gems:materials/block/ruby_ore"

[material]
base_color_texture = "example.gems:textures/block/ruby_ore"
normal_texture = "example.gems:textures/block/ruby_ore_normal"
fallback_debug_tint_rgba = "C02040FF"

[material.sampling]
profile = "voxel_nearest"
~~~

This texture authoring policy validates the referenced texture assets. The
material definition owns the surface meaning; the texture asset owns resource
bytes and image metadata.

## Relationship to asset overrides

Replacing texture bytes for the same texture key is an asset override decision.

Example:

~~~text
target:      freven.vanilla:textures/block/grass
replacement: example.green:textures/block/grass_lush
~~~

Rules:

- overriding a texture file is not the same as patching a material field;
- changing a material's `base_color_texture` is a content patch;
- replacing the bytes behind a texture key is an asset override;
- rebuilding atlases after either operation is generated cache behavior.

## Relationship to atlas / texture array pipeline

This document defines what texture assets are valid.

[TEXTURE_BACKEND_PIPELINE_v1.md](TEXTURE_BACKEND_PIPELINE_v1.md) defines how valid resolved textures become renderer
backend resources.

This document does not define:

- atlas page size;
- atlas packing order;
- texture-array layer index;
- padding/extrusion algorithm;
- runtime texture slot id;
- cache key layout;
- GPU upload strategy.

Those are generated backend details and belong to [TEXTURE_BACKEND_PIPELINE_v1.md](TEXTURE_BACKEND_PIPELINE_v1.md).

## Examples

### Accepted voxel block texture

~~~text
key:     example.gems:textures/block/ruby_ore
path:    assets/textures/block/ruby_ore.png
profile: voxel_block
size:    32x32
format:  PNG
sampling: voxel_nearest
~~~

### Rejected voxel block texture

~~~text
key:     example.gems:textures/block/ruby_ore_wide
path:    assets/textures/block/ruby_ore_wide.png
profile: voxel_block
size:    32x16
result:  rejected, voxel_block textures must be square
~~~

### Accepted UI texture

~~~text
key:     example.ui:textures/panel/header
path:    assets/textures/panel/header.png
profile: ui
size:    512x128
format:  PNG
sampling: ui_linear
~~~

### PBR model texture set

~~~text
example.space:textures/hull/panel_albedo -> sRGB color
example.space:textures/hull/panel_normal -> linear/data normal
example.space:textures/hull/panel_mr     -> linear/data metallic/roughness
~~~

## Long-term direction

Texture authoring v1 is conservative.

Future versions may add:

- explicit texture declaration files;
- richer metadata sidecars;
- KTX2/Basis authoring pipeline;
- supplied mip chains;
- animation sheets;
- texture arrays declared as authored groups while keeping layer ids internal;
- biome/color-map inputs;
- normal-map convention flags;
- product-specific maximum texture budgets;
- CI reports for texture memory estimates.

Those additions should extend stable author-facing policy without exposing
renderer internals.
