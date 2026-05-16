# Material Definitions v1

This document defines the Freven rc10 data-driven material definition schema.

It builds on:

- [ARCHITECTURE.md](ARCHITECTURE.md): engine / SDK / experience / mod / content-pack / standalone-product ownership;
- [PACKAGE_BOUNDARIES.md](PACKAGE_BOUNDARIES.md): manifest, config, content, assets, generated cache, and save/world state ownership;
- [VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md): stable visual asset identity and dependency model;
- [LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md): visual asset override decisions;
- [CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md): structured content add/replace/patch/disable semantics.

## Goals

- Define material declarations as author-facing content data.
- Keep renderer slots, palette ids, atlas coordinates, texture-array layers, GPU handles, Bevy handles, and generated cache paths out of authored material files.
- Give materials stable `namespace:path` keys.
- Let materials reference texture assets by stable visual asset keys.
- Support a small PBR-ready surface model without requiring a full material graph.
- Preserve a simple fallback/debug path for the current debug-palette renderer.
- Keep Vanilla material libraries in Vanilla or standalone content, not in engine code.
- Leave texture size/sampling/atlas implementation details to dedicated rc10 documents.

## Non-goals

This document does not define:

- texture dimensions, file formats, mipmap generation, or filtering policy, defined by [TEXTURE_AUTHORING_v1.md](TEXTURE_AUTHORING_v1.md);
- texture atlas or texture-array packing;
- renderer-internal material table layout;
- Bevy/wgpu material APIs;
- shader graph authoring;
- arbitrary renderer plugin ABI;
- final item/entity visual binding schemas;
- block visual binding schema, defined by
  [BLOCK_VISUAL_DEFINITIONS_v1.md](BLOCK_VISUAL_DEFINITIONS_v1.md);
- Vanilla's actual stone/dirt/grass material library.

Those belong to separate SDK, engine, Vanilla, and DevKit issues.

## Core rule

A material is content data.

A material definition describes a renderable surface in stable author-facing terms. It is not a texture file, not a renderer slot, not an atlas region, not a GPU handle, and not generated cache.

Author-facing material:

~~~toml
schema = 1
key = "example.gems:materials/block/ruby_ore"

[material]
base_color_texture = "example.gems:textures/block/ruby_ore"
fallback_debug_tint_rgba = "C02040FF"
alpha_mode = "opaque"
render_layer = "opaque"
lighting_model = "lit"
~~~

Host/backend output:

~~~text
renderer material slot
texture array layer
atlas coordinates
GPU bind group
shader pipeline id
Bevy Handle<Image>
generated cache path
~~~

Only the first model is public SDK vocabulary.

## Ownership and location

Material definitions live in content data, normally under:

~~~text
content/materials/
~~~

Texture source files live under assets, normally under:

~~~text
assets/textures/
~~~

Generated atlases, texture arrays, transcodes, fingerprints, and load plans live in generated cache and are rebuildable.

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
        ruby_ore_n.png
        ruby_ore_mr.png
~~~

## Stable material keys

A material key is a stable visual asset key:

~~~text
namespace:path
~~~

Examples:

~~~text
freven.vanilla:materials/block/stone
freven.vanilla:materials/block/grass
example.gems:materials/block/ruby_ore
example.space:materials/hull/panel
example.ui:materials/icon/warning
~~~

Rules:

- the namespace owns the material identity;
- the path identifies the material inside that namespace;
- keys should be stable across compatible releases;
- keys must not encode renderer slots, runtime ids, atlas coordinates, texture array layers, GPU handles, or cache paths;
- one selected stack must resolve each effective material key to one material declaration;
- replacing or patching a material key is structured content behavior, not raw file shadowing.

The accepted rc10 key shape follows the existing Freven `namespace:path` pattern.

## Material schema v1

Canonical material files use:

~~~toml
schema = 1
key = "example.gems:materials/block/ruby_ore"

[material]
base_color_texture = "example.gems:textures/block/ruby_ore"
fallback_debug_tint_rgba = "C02040FF"
alpha_mode = "opaque"
render_layer = "opaque"
lighting_model = "lit"
tint_rgba = "FFFFFFFF"
~~~

### Required fields

| Field | Type | Meaning |
| --- | --- | --- |
| `schema` | integer | Material file schema version. v1 uses `1`. |
| `key` | `namespace:path` | Stable material identity. |
| `material.fallback_debug_tint_rgba` | `RRGGBBAA` hex string | Visible fallback color while real material rendering is unavailable or while an asset is missing. |

### Common fields

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `material.base_color_texture` | texture key | none | Main albedo/base-color texture. |
| `material.base_color_rgba` | `RRGGBBAA` hex string | `"FFFFFFFF"` | Constant base color used when no texture is present or multiplied with the texture. |
| `material.tint_rgba` | `RRGGBBAA` hex string | `"FFFFFFFF"` | Runtime/editor tint multiplier. |
| `material.alpha_mode` | enum | `"opaque"` | Alpha behavior. |
| `material.alpha_cutoff` | number `0.0..1.0` | `0.5` for cutout | Cutout threshold. Ignored for opaque. |
| `material.render_layer` | enum | derived from `alpha_mode` | Coarse render bucket requested by authoring. |
| `material.lighting_model` | enum | `"lit"` | Lighting/shading behavior. |

### PBR-ready optional fields

| Field | Type | Meaning |
| --- | --- | --- |
| `material.normal_texture` | texture key | Tangent-space normal map. |
| `material.metallic` | number `0.0..1.0` | Constant metallic factor. |
| `material.roughness` | number `0.0..1.0` | Constant roughness factor. |
| `material.metallic_roughness_texture` | texture key | Packed metallic/roughness map. Channel packing is defined by texture policy docs. |
| `material.emissive_texture` | texture key | Emissive color map. |
| `material.emissive_rgba` | `RRGGBBAA` hex string | Constant emissive color. |
| `material.emissive_strength` | number `>= 0.0` | Emissive multiplier. |
| `material.occlusion_texture` | texture key | Ambient occlusion map. Optional and renderer-dependent. |

v1 is PBR-ready, not a full material graph. Unsupported optional fields must produce diagnostics or deterministic fallback behavior.

## Enums

### `alpha_mode`

Accepted canonical values:

| Value | Meaning |
| --- | --- |
| `opaque` | Fully opaque material. Alpha is ignored for sorting/blending. |
| `cutout` | Binary alpha test material, such as leaves, grass, glass panes, fences, decals, or sprites. |
| `blend` | Transparent/blended material. Renderer may sort/bucket it separately. |

Rules:

- `opaque` materials should normally use `render_layer = "opaque"`;
- `cutout` materials should normally use `render_layer = "cutout"`;
- `blend` materials should normally use `render_layer = "transparent"`;
- mismatched `alpha_mode` and `render_layer` should be a warning or error depending on authoring strictness.

### `render_layer`

Accepted canonical values:

| Value | Meaning |
| --- | --- |
| `opaque` | Opaque render bucket. |
| `cutout` | Alpha-tested render bucket. |
| `transparent` | Blended/translucent render bucket. |

Creator-friendly aliases may compile to canonical values:

| Alias | Canonical |
| --- | --- |
| `solid` | `opaque` |
| `alpha_test` | `cutout` |
| `translucent` | `transparent` |

Canonical stored content should use `opaque`, `cutout`, or `transparent`.

### `lighting_model`

Accepted v1 values:

| Value | Meaning |
| --- | --- |
| `lit` | Default engine-lit material. |
| `unlit` | Ignores scene lighting; useful for UI, debug, decals, or emissive-only effects. |
| `pbr_lit` | PBR-ready lit material using metallic/roughness-style fields where supported. |

The current renderer may treat `lit` and `pbr_lit` similarly until the material pipeline matures. The schema still records the author's intent.

## Texture references

Material texture fields reference visual texture asset keys:

~~~toml
base_color_texture = "example.gems:textures/block/ruby_ore"
normal_texture = "example.gems:textures/block/ruby_ore_n"
metallic_roughness_texture = "example.gems:textures/block/ruby_ore_mr"
~~~

Rules:

- texture references are stable keys, not file paths and not renderer slots;
- missing texture keys are diagnostics;
- wrong asset type is a diagnostic;
- texture file path, dimensions, format, mip policy, sampling policy, color space, compression, and atlas packing are not defined by this document;
- generated atlas coordinates or texture-array layers must never appear in a material definition.

Texture size, sampling, and validation rules are defined by [TEXTURE_AUTHORING_v1.md](TEXTURE_AUTHORING_v1.md). Atlas/array packing is defined by [TEXTURE_BACKEND_PIPELINE_v1.md](TEXTURE_BACKEND_PIPELINE_v1.md).

## Sampling reference

Material v1 may optionally include a sampling profile reference:

~~~toml
[material.sampling]
profile = "voxel_nearest"
~~~

Rules:

- `sampling.profile` is an author-facing policy name, not a sampler object or GPU handle;
- the accepted profiles and exact filtering/mipmap behavior belong to [TEXTURE_AUTHORING_v1.md](TEXTURE_AUTHORING_v1.md);
- unknown profiles are diagnostics;
- when omitted, tooling chooses the default profile for the material context.

This lets material definitions express intent without baking renderer internals into content.

## Examples

### Debug-only material

~~~toml
schema = 1
key = "example.debug:materials/block/test"

[material]
fallback_debug_tint_rgba = "FF00FFFF"
alpha_mode = "opaque"
render_layer = "opaque"
lighting_model = "lit"
~~~

### Simple opaque block material

~~~toml
schema = 1
key = "example.gems:materials/block/ruby_ore"

[material]
base_color_texture = "example.gems:textures/block/ruby_ore"
fallback_debug_tint_rgba = "C02040FF"
alpha_mode = "opaque"
render_layer = "opaque"
lighting_model = "lit"
tint_rgba = "FFFFFFFF"
~~~

### Cutout leaves

~~~toml
schema = 1
key = "example.trees:materials/block/maple_leaves"

[material]
base_color_texture = "example.trees:textures/block/maple_leaves"
fallback_debug_tint_rgba = "3FA34DCC"
alpha_mode = "cutout"
alpha_cutoff = 0.5
render_layer = "cutout"
lighting_model = "lit"
~~~

### Transparent glass

~~~toml
schema = 1
key = "example.glass:materials/block/blue_glass"

[material]
base_color_texture = "example.glass:textures/block/blue_glass"
base_color_rgba = "80BFFFFF"
fallback_debug_tint_rgba = "80BFFFFF"
alpha_mode = "blend"
render_layer = "transparent"
lighting_model = "lit"
roughness = 0.05
metallic = 0.0
~~~

### PBR-ready metal panel

~~~toml
schema = 1
key = "example.space:materials/hull/panel"

[material]
base_color_texture = "example.space:textures/hull/panel_albedo"
normal_texture = "example.space:textures/hull/panel_normal"
metallic_roughness_texture = "example.space:textures/hull/panel_mr"
fallback_debug_tint_rgba = "606870FF"
alpha_mode = "opaque"
render_layer = "opaque"
lighting_model = "pbr_lit"
metallic = 1.0
roughness = 0.45
~~~

### Emissive firefly

~~~toml
schema = 1
key = "example.creatures:materials/entity/firefly"

[material]
base_color_texture = "example.creatures:textures/entity/firefly"
emissive_texture = "example.creatures:textures/entity/firefly_emissive"
emissive_rgba = "FFF080FF"
emissive_strength = 2.5
fallback_debug_tint_rgba = "FFF080FF"
alpha_mode = "blend"
render_layer = "transparent"
lighting_model = "lit"
~~~

## Friendly authoring layer

Creator-friendly files may allow shorthand:

~~~toml
texture = "textures/block/ruby_ore"
fallback_debug_tint_rgba = "C02040FF"
~~~

Tooling may expand this to canonical material content:

~~~toml
key = "example.gems:materials/block/ruby_ore"
base_color_texture = "example.gems:textures/block/ruby_ore"
~~~

Rules:

- shorthand is tooling input, not the canonical resolved identity;
- canonical content stores stable visual asset keys;
- generated atlas/array ids are never valid shorthand output;
- friendly defaults must be deterministic and explainable by DevKit diagnostics.

## Relationship to block descriptors

Current block descriptors can reference a material key:

~~~rust
BlockDescriptor::solid_material_cube(
    "example.gems:materials/block/ruby_ore",
    0xC02040FF,
)
~~~

The material key is the stable author-facing identity. The fallback debug tint keeps the block visible before the renderer has full texture/material support.

Rules:

- the block descriptor does not own the material definition;
- the material definition lives in content data;
- the host resolves the material key through the selected content/material stack;
- renderer palette, atlas, texture-array, and material-table slots remain host-internal;
- the material fallback color and block fallback color should match when they describe the same material-key block visual.

## Dependency graph

A material declaration may depend on:

~~~text
material -> base_color_texture
material -> normal_texture
material -> metallic_roughness_texture
material -> emissive_texture
material -> occlusion_texture
material -> sampling profile
material -> effect/shader preset (future)
~~~

Rules:

- dependencies must resolve before runtime use;
- missing dependencies produce diagnostics;
- wrong asset kind produces diagnostics;
- dependency order must be deterministic;
- diagnostics should include the material key, field name, referenced key, owner layer, and source file.

## Compatibility and fingerprints

For server-required or authoritative visual content, the material declaration fingerprint should include:

- material key;
- schema version;
- alpha mode;
- render layer;
- lighting model;
- texture dependency keys;
- scalar factors and color fields;
- fallback debug tint;
- sampling profile reference, if present;
- declared compatibility class or policy metadata, when added by future docs.

The fingerprint should not include:

- install path;
- filesystem walk order;
- renderer slot id;
- atlas coordinate;
- texture-array layer;
- Bevy/wgpu handle;
- generated cache path.

Texture bytes and image metadata are handled by texture/asset policy documents. Generated atlas/load-plan fingerprints are derived output, not authored material truth.

## Patch and override behavior

Material definitions are content entries.

Use [CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md) for semantic operations such as:

~~~toml
kind = "material"
target = "freven.vanilla:materials/block/stone"
op = "patch"
path = "roughness"
value = 0.85
~~~

Use [LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md) for raw visual asset replacement decisions such as swapping a texture asset key.

Rules:

- patching `roughness` is a material content patch;
- replacing `base_color_texture` with another texture key is a material content patch;
- replacing the bytes behind a texture key is an asset override decision;
- rebuilding an atlas after either change is generated cache behavior;
- no patch may write renderer slots, atlas coordinates, GPU handles, or cache paths into material content.

## Validation diagnostics

Validators should report at least:

| Diagnostic | Should include |
| --- | --- |
| Invalid material key | file path, key, expected `namespace:path` |
| Duplicate material key | key, winner layer, shadowed/conflicting layer |
| Missing texture key | material key, field, referenced texture key |
| Wrong asset kind | material key, field, referenced key, expected texture |
| Invalid color | material key, field, expected `RRGGBBAA` |
| Invalid enum | material key, field, accepted values |
| Invalid numeric range | material key, field, accepted range |
| Alpha/render-layer mismatch | material key, alpha mode, render layer |
| Unsupported optional PBR field | material key, field, runtime capability |
| Renderer-internal id used | material key, field, forbidden value |
| Missing fallback debug tint | material key, file path |
| Unknown sampling profile | material key, profile, accepted profiles |

Diagnostics should be actionable for DevKit users and should not require knowing engine internals.

## Current engine bridge

The current engine transition path can compile a subset of this schema into the existing Material Registry v1 shape:

| Material schema v1 | Current engine bridge |
| --- | --- |
| `key` | `MaterialDescriptorV1.key` |
| `base_color_texture` | current `texture_key` |
| `fallback_debug_tint_rgba` | current fallback debug tint |
| texture declaration path/hash | current texture descriptor / manifest validation |

Other fields are preserved as author-facing schema intent for the real material pipeline and renderer work. They must not be silently exposed as renderer-local ids.

## Relationship to follow-up issues

This document intentionally leaves these details to follow-up work:

- texture size, format, filtering, mipmap, color-space, and sampling validation, defined by [TEXTURE_AUTHORING_v1.md](TEXTURE_AUTHORING_v1.md);
- atlas or texture-array packing, defined by [TEXTURE_BACKEND_PIPELINE_v1.md](TEXTURE_BACKEND_PIPELINE_v1.md);
- material table GPU layout;
- renderer backend bind groups and shader modules;
- Vanilla material library content;
- full block/item/entity visual binding schema.

## Long-term direction

Material v1 is deliberately small. It gives mod authors and standalone games a stable material definition language while allowing the engine renderer to evolve.

Future versions may add:

- named material presets;
- shader/effect references;
- per-face block material bindings;
- material variants;
- biome/color-map inputs;
- animation parameters;
- layered materials;
- clearer physically based parameterization;
- explicit client-local cosmetic override policy.

Those additions should extend the author-facing schema without exposing renderer internals.
