# Block Visual Definitions v1

This document defines the Freven rc10 data-driven block visual definition schema.

It builds on:

- [ARCHITECTURE.md](ARCHITECTURE.md): engine / SDK / experience / mod /
  content-pack / standalone-product ownership;
- [PACKAGE_BOUNDARIES.md](PACKAGE_BOUNDARIES.md): manifest, config, content,
  assets, generated cache, and save/world state ownership;
- [VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md): stable visual asset
  identity, visual graph resolution, validation, and renderer-internal
  boundaries;
- [MATERIAL_DEFINITIONS_v1.md](MATERIAL_DEFINITIONS_v1.md): data-driven
  material definitions;
- [TEXTURE_AUTHORING_v1.md](TEXTURE_AUTHORING_v1.md): texture size, sampling,
  mipmap, alpha, and validation policy;
- [TEXTURE_BACKEND_PIPELINE_v1.md](TEXTURE_BACKEND_PIPELINE_v1.md): generated
  atlas/texture-array/backend planning and renderer-internal slot boundaries;
- [LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md): deterministic
  visual asset layering and override policy;
- [CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md): semantic add,
  replace, patch, append, disable, compatibility, and diagnostics model;
- [CREATOR_CONTENT_SCHEMA_v1.md](CREATOR_CONTENT_SCHEMA_v1.md): creator-facing
  source schema direction;
- [DATA_DRIVEN_AUTHORING_LAYER_v1.md](DATA_DRIVEN_AUTHORING_LAYER_v1.md):
  practical authoring workflow and shorthand expansion;
- [SHADER_EFFECT_BOUNDARY_v1.md](SHADER_EFFECT_BOUNDARY_v1.md): shader/effect
  ownership boundary for visual effect references, capabilities, fallbacks, trust,
  and renderer-internal resource boundaries.

The goal is to define how gameplay block content is bound to authored visuals
without exposing renderer slots, atlas coordinates, runtime block ids, or
Vanilla-specific shortcuts.

## Core rule

Block visuals are content data.

A block visual definition binds a gameplay block key to visual assets such as
models, materials, tint sources, and render policy. It does not define gameplay
behavior, runtime block ids, renderer material slots, atlas coordinates,
texture-array layers, GPU handles, or generated cache paths.

Author-facing model:

~~~toml
schema = 1
key = "example.gems:visuals/block/ruby_ore"

[visual]
target = "example.gems:blocks/ruby_ore"
model = "example.gems:models/block/cube_all"

[visual.materials]
all = "example.gems:materials/block/ruby_ore"
~~~

Host/backend output:

~~~text
runtime block id
renderer material slot
atlas page and rectangle
texture array layer
chunk mesh section/layer
GPU bind group
Bevy/wgpu handle
generated cache artifact
~~~

Only the first model is public SDK vocabulary.

## Goals

- Keep gameplay block definitions separate from visual definitions.
- Let blocks reference cube, per-face, and reusable model visuals through stable
  namespaced keys.
- Let visuals reference materials by stable material keys.
- Support simple all-sides cube blocks without requiring custom model files.
- Support per-face materials for grass, logs, ores, machines, and similar blocks.
- Support model references for non-trivial block geometry.
- Keep collision and selection boxes separate from visual mesh geometry.
- Preserve tint/color-map hooks for later biome/variant/color systems.
- Preserve render-layer intent without making renderer backend state public.
- Support deterministic composition for Vanilla, mods, content packs, total
  conversions, and zero-Vanilla standalone games.
- Provide a clean target for engine block model meshing and DevKit validation.

## Non-goals

This document does not define:

- the model asset format, defined by
  [MODEL_ASSET_FORMAT_v1.md](MODEL_ASSET_FORMAT_v1.md);
- the content variant/family expansion schema, defined by
  [CONTENT_VARIANT_FAMILY_SCHEMA_v1.md](CONTENT_VARIANT_FAMILY_SCHEMA_v1.md);
- the final engine meshing implementation;
- arbitrary glTF/static mesh/entity animation support;
- final lighting, tint/color-map, or shader extension implementation;
- final DevKit commands or UI;
- Vanilla's actual block visual library.

Those are separate rc10 SDK, engine, DevKit, and Vanilla issues.

## Terminology

| Term | Meaning |
| --- | --- |
| Block entry | Gameplay/content definition for a block key. |
| Block visual | Content entry that binds a block key to visual assets and visual policy. |
| Model | Reusable geometry/layout declaration referenced by a block visual. |
| Material | Surface declaration referenced by a block visual or model material slot. |
| Material slot | Author-facing named slot inside a model or block visual, not a renderer slot. |
| Render layer | Author-facing coarse render bucket such as opaque, cutout, or transparent. |
| Collision shape | Gameplay/physics shape used by world collision. |
| Selection shape | Interaction/raycast/highlight shape used by tools and UI. |
| Runtime block id | Host/runtime compact id. Not authored content identity. |
| Renderer slot | Host/renderer compact material or texture slot. Not authored content identity. |

## Stable keys

A block visual has a stable content key:

~~~text
namespace:visuals/block/name
~~~

Examples:

~~~text
freven.vanilla:visuals/block/stone
freven.vanilla:visuals/block/grass_block
freven.vanilla:visuals/block/glass
example.gems:visuals/block/ruby_ore
example.machines:visuals/block/crusher
~~~

Rules:

- the namespace owns the visual identity;
- the visual key is separate from the gameplay block key;
- the visual targets a block key explicitly;
- keys must not encode runtime block ids, renderer slots, atlas coordinates,
  texture-array layers, or generated cache paths;
- the selected content graph must resolve one effective visual binding for each
  block where a required visual is needed;
- replacing or patching a block visual uses content patch/merge semantics.

## Ownership and location

Block visual definitions live in content data, normally under:

~~~text
content/visuals/
~~~

A simple package layout may look like:

~~~text
mods/example.gems/
  mod.toml
  content/
    blocks/
      ruby_ore.toml
    visuals/
      block/
        ruby_ore.toml
    materials/
      block/
        ruby_ore.toml
    models/
      block/
        cube_all.toml
  assets/
    textures/
      block/
        ruby_ore.png
~~~

A creator-friendly block file may also embed a `[visual]` section. That section
is source syntax. The authoring layer compiles it into semantic block visual,
material, and model entries where needed.

## Block visual schema v1

Canonical block visual files use:

~~~toml
schema = 1
key = "example.gems:visuals/block/ruby_ore"

[visual]
target = "example.gems:blocks/ruby_ore"
model = "example.gems:models/block/cube_all"
render_layer = "opaque"
fallback_debug_tint_rgba = "C02040FF"

[visual.materials]
all = "example.gems:materials/block/ruby_ore"
~~~

### Required fields

| Field | Type | Meaning |
| --- | --- | --- |
| `schema` | integer | Block visual file schema version. v1 uses `1`. |
| `key` | `namespace:path` | Stable block visual identity. |
| `visual.target` | block key | Gameplay block key this visual binds to. |

### Common fields

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `visual.model` | model key | selected policy default, if allowed | Reusable model/layout key. |
| `visual.material` | material key | none | Simple single-material shorthand. |
| `visual.materials` | map of slot -> material key | none | Named material slot bindings. |
| `visual.texture` | texture key | none | Friendly shorthand that may compile to a material/model binding. |
| `visual.render_layer` | enum | derived from material/model | Coarse render bucket. |
| `visual.tint_rgba` | `RRGGBBAA` | `FFFFFFFF` | Constant tint multiplier. |
| `visual.tint_source` | enum/string | none | Future biome/color-map/variant tint source. |
| `visual.fallback_debug_tint_rgba` | `RRGGBBAA` | material fallback if available | Visible fallback color. |
| `visual.variant_selector` | string/key | none | Hook into resolved variant/family data. |

The exact model fields, cuboid parts, UVs, transforms, and material-slot rules are
defined by [MODEL_ASSET_FORMAT_v1.md](MODEL_ASSET_FORMAT_v1.md). This document
only defines how block visuals bind gameplay blocks to visual assets.

## Simple cube visual

A simple all-sides cube can be expressed with one material.

~~~toml
schema = 1
key = "example.gems:visuals/block/ruby_block"

[visual]
target = "example.gems:blocks/ruby_block"
model = "freven.core:models/block/cube_all"
material = "example.gems:materials/block/ruby_block"
render_layer = "opaque"
fallback_debug_tint_rgba = "C02040FF"
~~~

The selected experience or product may provide `freven.core:models/block/cube_all`
or a different default model library. The engine must not hardcode Vanilla's
visual style.

## Per-face cube visual

Per-face materials are expressed through named material slots.

~~~toml
schema = 1
key = "freven.vanilla:visuals/block/grass_block"

[visual]
target = "freven.vanilla:blocks/grass_block"
model = "freven.core:models/block/cube_faces"
render_layer = "opaque"
fallback_debug_tint_rgba = "6BAA3AFF"

[visual.materials]
top = "freven.vanilla:materials/block/grass_top"
bottom = "freven.vanilla:materials/block/dirt"
north = "freven.vanilla:materials/block/grass_side"
south = "freven.vanilla:materials/block/grass_side"
east = "freven.vanilla:materials/block/grass_side"
west = "freven.vanilla:materials/block/grass_side"
~~~

Canonical face names for cube-style block visuals are:

| Face | Meaning |
| --- | --- |
| `top` | Positive Y face. |
| `bottom` | Negative Y face. |
| `north` | Negative Z face. |
| `south` | Positive Z face. |
| `east` | Positive X face. |
| `west` | Negative X face. |
| `all` | Shorthand for all faces where the model supports it. |
| `side` | Shorthand for north/south/east/west where the model supports it. |

A model schema may define additional named slots. Unknown slots are validation
errors unless the referenced model explicitly declares extension slots.

## Model-backed visual

A block visual may reference a reusable model with named material slots.

~~~toml
schema = 1
key = "example.glass:visuals/block/framed_glass"

[visual]
target = "example.glass:blocks/framed_glass"
model = "example.glass:models/block/framed_glass"
render_layer = "transparent"
fallback_debug_tint_rgba = "80BFFFFF"

[visual.materials]
frame = "example.glass:materials/block/iron_frame"
pane = "example.glass:materials/block/blue_glass"
~~~

The model owns geometry, cuboids, UVs, and material slot declarations. The block
visual binds those slots to effective material keys.

## Friendly texture shorthand

Beginner authoring may allow a block visual to reference a texture directly:

~~~toml
schema = 1
key = "blocks/ruby_ore"

[visual]
texture = "textures/block/ruby_ore"
fallback_debug_tint_rgba = "C02040FF"
~~~

This is source syntax only. The authoring layer expands it into stable semantic
entries such as:

~~~text
material:example.gems:materials/block/ruby_ore
model:example.gems:models/block/ruby_ore or selected default cube model
visual:example.gems:visuals/block/ruby_ore
~~~

The resolved graph still contains material/model/visual keys. Renderer slots are
not authored.

## Render layers

Accepted canonical render-layer values are:

| Value | Meaning |
| --- | --- |
| `opaque` | Fully opaque terrain/block bucket. |
| `cutout` | Alpha-tested bucket for leaves, panes, fences, decals, sprites, and similar visuals. |
| `transparent` | Blended/translucent bucket. |

Creator-friendly aliases may compile to canonical values:

| Alias | Canonical |
| --- | --- |
| `solid` | `opaque` |
| `alpha_test` | `cutout` |
| `translucent` | `transparent` |

Rules:

- if a visual declares `render_layer`, it must be compatible with the resolved
  material alpha modes;
- material alpha mode normally derives the final render bucket;
- mismatches should be diagnostics;
- transparent/cutout visuals must not be used as a workaround for gameplay
  solidity or collision.

## Tint and color maps

Block visual v1 reserves tint fields for future biome, color-map, and variant
systems.

~~~toml
[visual]
tint_rgba = "FFFFFFFF"
tint_source = "biome_grass"
~~~

Rules:

- `tint_rgba` is a constant author-facing multiplier;
- `tint_source` names a future color source or color-map policy;
- unsupported tint sources must produce diagnostics or deterministic fallback;
- tint fields must not encode renderer uniform buffer slots, texture atlas
  coordinates, or shader internals.

Full biome/color-map behavior belongs to tint/color-map pipeline work.

## Collision and selection are not visuals

Visual mesh geometry is not gameplay collision.

A block may visually look like a thin pane, tall grass, a slab, a frame, or a
multi-part machine while using a separate collision and selection policy.

Rules:

- collision shape affects gameplay/physics and is authoritative where the
  selected experience/server policy requires it;
- selection shape affects raycast/highlight/tool interaction;
- visual model geometry affects rendering only;
- changing visual model geometry must not silently change collision;
- changing collision or selection uses block/gameplay/content schema, not raw
  visual asset override;
- client-local cosmetic packs may not change collision or selection.

Conceptual split:

~~~toml
schema = 1
key = "example.glass:blocks/framed_glass"

[block]
solid = true
opaque = false

[collision]
kind = "full_block"

[selection]
kind = "model_bounds"

[visual]
model = "example.glass:models/block/framed_glass"
~~~

The final collision/selection schema is owned by block/gameplay content work. The
block visual contract only requires the separation.

## Variants

Block visual v1 supports variant hooks but does not define the full family
expansion system.

Conceptual example:

~~~toml
schema = 1
key = "example.woods:visuals/block/planks"

[visual]
target = "example.woods:blocks/planks"
variant_selector = "example.woods:families/wood_species"
model = "freven.core:models/block/cube_all"

[visual.variant_materials.oak]
all = "example.woods:materials/block/oak_planks"

[visual.variant_materials.willow]
all = "example.woods:materials/block/willow_planks"
~~~

Rules:

- generated variant keys must be deterministic;
- diagnostics should show generated block, material, model, and visual keys;
- runtime block ids remain internal;
- variant axes, skip/allow lists, per-variant overrides, and generated-key policy
  are defined by
  [CONTENT_VARIANT_FAMILY_SCHEMA_v1.md](CONTENT_VARIANT_FAMILY_SCHEMA_v1.md).

## Composition and patching

Block visuals compose through content patch/merge semantics.

Examples:

~~~toml
[[content_patches]]
op = "patch"
kind = "visual"
target = "freven.vanilla:visuals/block/stone"
path = "visual.material"
value = "example.pack:materials/block/stone_polished"
authority = "selected_stack"
reason = "selected visual refresh"
~~~

~~~toml
[[content_patches]]
op = "patch"
kind = "visual"
target = "freven.vanilla:visuals/block/grass_block"
path = "visual.materials.top"
value = "example.pack:materials/block/grass_top_autumn"
authority = "selected_stack"
reason = "seasonal visual pack"
~~~

Rules:

- replacing or patching block visuals must be explicit;
- accidental duplicate visual keys are conflicts;
- patching one material slot is structured content patching, not raw file
  shadowing;
- visual patches must not write renderer slots, runtime ids, atlas coordinates,
  generated cache paths, collision, selection, or gameplay state;
- client-local cosmetic policy may allow texture/material swaps but must not
  silently replace server-required visual bindings.

## Authority and compatibility

Block visuals can be cosmetic, server-required, or authoritative depending on
selected product/experience/server policy.

| Class | Meaning |
| --- | --- |
| Cosmetic visual | Local presentation-only change allowed by policy. |
| Server-required visual binding | Required effective visual binding or accepted equivalent for compatibility/presentation. |
| Authoritative visual binding | Visual binding that participates in selected stack identity. |
| Gameplay content | Block behavior, collision, selection, drops, hardness, provider links, or save/world meaning. |

Rules:

- gameplay block identity is separate from visual binding identity;
- required visual bindings may participate in compatibility fingerprints;
- cosmetic packs may change allowed texture/material assets but cannot change
  gameplay meaning;
- changing collision, selection, drops, provider behavior, or save/world state is
  not a visual override;
- diagnostics should explain whether a block visual is cosmetic, selected-stack,
  server-required, or authoritative.

## Validation

Block visual validation should happen before runtime start.

Validation should catch:

- missing `schema`;
- unsupported schema version;
- missing or invalid visual key;
- missing or invalid `visual.target`;
- unresolved target block key;
- unresolved model key;
- unresolved material key;
- material slot not declared by the referenced model;
- required material slot missing;
- invalid render-layer value;
- render-layer/material alpha mismatch;
- invalid tint value;
- unsupported tint source;
- duplicate block visual key;
- duplicate effective visual binding for one target block without explicit
  replace/patch policy;
- renderer-internal id in authored data;
- runtime block id in authored data;
- atlas coordinate or texture-array layer in authored data;
- generated cache referenced as authored source;
- visual patch attempting to modify collision, selection, or gameplay fields.

Diagnostics should report:

- file path;
- line/field path where possible;
- visual key;
- target block key;
- referenced model/material/texture key;
- owner package/layer;
- selected override/patch operation;
- expected type or allowed values;
- suggested fix.

## Relationship to current material-key block descriptors

The current runtime bridge already supports material-key block visuals through
stable material key hashes and fallback debug tints.

That bridge is a compatibility path, not the final authoring surface.

Rules:

- material-key block descriptors remain useful for runtime-loaded Wasm guests and
  transition-era content;
- data-driven block visuals are the long-term authoring model;
- material key hashes and renderer material slots are host/runtime details;
- authors should use stable material/model/visual keys in content data;
- fallback debug tint must remain visible and diagnosable while real assets are
  missing or unsupported.

## Relationship to model format v1

See [MODEL_ASSET_FORMAT_v1.md](MODEL_ASSET_FORMAT_v1.md).

This document defines how a block visual references a model and binds material
slots.

The model format defines:

- cube model declarations;
- per-face cube model declarations;
- cuboid parts;
- UV mapping;
- transforms, origins, and rotations;
- model-local material slot declarations;
- future item/entity/static mesh compatibility.

If there is disagreement, the model format owns geometry and material-slot
declaration, while this document owns block-to-visual binding.

## Relationship to variants/families

See [CONTENT_VARIANT_FAMILY_SCHEMA_v1.md](CONTENT_VARIANT_FAMILY_SCHEMA_v1.md).

This document reserves variant hooks and shows how variant-specific visual fields
can be represented.

The variant/family expansion schema defines:

- variant axes;
- generated keys;
- skip/allow lists;
- per-variant overrides;
- generated block/material/model/visual entries;
- diagnostics for invalid combinations.

If there is disagreement, family expansion owns generated entry creation, while
this document owns the shape of each resulting block visual entry.

## DevKit guidance

DevKit should eventually be able to:

- validate block visual files;
- explain which block a visual targets;
- show resolved model/material/texture dependencies;
- show effective per-face material bindings;
- show whether a visual is cosmetic, selected-stack, server-required, or
  authoritative;
- show patch/override provenance for each visual field;
- report collision/selection edits as non-visual gameplay changes;
- show renderer-internal atlas/slot data only in inspector/debug views, never as
  authoring fields;
- generate starter visuals for simple cube, per-face cube, glass/frame, slab, and
  model-backed blocks.

## Summary

Block visual definitions are the stable author-facing bridge between gameplay
blocks and visual assets.

They bind block keys to model/material/tint/render policy while keeping gameplay
logic, collision, selection, runtime block ids, renderer slots, atlases, GPU
handles, and generated cache out of authored visual data.

## Relationship to lighting foundation

Lighting behavior for block visuals is defined by
[LIGHTING_FOUNDATION_v1.md](LIGHTING_FOUNDATION_v1.md).

Rules:

- render layer does not silently define light opacity or transmission;
- transparent and cutout visuals should declare explicit light behavior or inherit
  selected-stack policy;
- per-face shading and AO are visual policy, not gameplay collision;
- emitted block light belongs to explicit light metadata, not just emissive
  material appearance;
- shader uniforms, vertex attributes, lightmap coordinates, and renderer light
  handles must not appear in block visual files.

## Relationship to shader/effect boundary

Block visuals may reference named effects, but shader/effect ownership is defined
by [SHADER_EFFECT_BOUNDARY_v1.md](SHADER_EFFECT_BOUNDARY_v1.md).

Rules:

- visual effect references use stable effect keys;
- render layer, tint, lighting, and effect policy are semantic visual data;
- shader uniforms, vertex attribute locations, bind groups, pipeline ids, and
  GPU handles must not appear in block visual files;
- selected stack policy decides whether an effect is cosmetic, server-required,
  denied, or trusted.

## Conformance fixtures

Canonical block visual examples live under
`fixtures/visual_content_schema_v1/valid/`.

The fixture set includes `cube_all`, `cube_faces`, model-backed framed glass,
grass tint metadata, emissive/light metadata, and generated family examples.
Follow-up engine, DevKit, Boot, and Vanilla work should reference those fixtures
instead of inventing separate visual examples.
