# Visual Asset Model v1

This document defines the Freven visual asset model used by rc10 visual/data
foundation work.

It builds on:

- [ARCHITECTURE.md](ARCHITECTURE.md): engine / SDK / experience / mod /
  content-pack / standalone-product ownership;
- [PACKAGE_BOUNDARIES.md](PACKAGE_BOUNDARIES.md): manifest, config, content,
  assets, generated cache, and save/world state ownership.

This document defines stable visual asset identity, asset categories,
dependencies, validation, resolution, compatibility classes, and the boundary
between author-facing assets and renderer-internal backend handles.

## Goals

- Give authors stable `namespace:path` visual asset keys.
- Define the core visual asset categories: textures, materials, models,
  shaders/effects, and generated atlas/array/load-plan outputs.
- Keep renderer slots, atlas coordinates, texture-array indexes, Bevy handles,
  wgpu handles, and cache paths out of author-facing APIs.
- Make dependencies explicit and diagnosable.
- Allow Vanilla, Vanilla mods, content packs, total conversions, and zero-Vanilla
  standalone games to use the same model.
- Support deterministic asset resolution across client/server where assets are
  authoritative or server-required.
- Keep Vanilla visual style out of the engine.

## Non-goals

This document does not define:

- the full material schema, defined by [MATERIAL_DEFINITIONS_v1.md](MATERIAL_DEFINITIONS_v1.md);
- the full block visual schema;
- the final model asset format;
- the texture atlas or texture-array packing algorithm;
- the layered asset override algorithm;
- the content patch/merge algorithm defined by [CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md);
- shader plugin ABI or arbitrary renderer extension APIs;
- final DevKit implementation details.

Those are separate rc10 documents and implementation issues.

## Core rule

Authors refer to visual assets by stable keys and typed declarations.

The host resolves those keys into renderer-internal handles, cache entries,
atlases, texture arrays, GPU resources, or backend-specific objects.

Author-facing model:

```text
example.mod:textures/block/ruby_ore
example.mod:materials/ruby_ore
example.mod:models/block/ruby_ore
example.mod:effects/wind_leaves
```

Host-internal model:

```text
atlas page
texture array layer
GPU bind group
material pipeline id
Bevy handle
wgpu texture/view/sampler
compiled shader module
generated load-plan node
```

Only the first model is stable public SDK vocabulary. The second model is
implementation detail.

## Stable asset keys

A visual asset key is a stable namespaced identity:

```text
namespace:path
```

Examples:

```text
freven.vanilla:textures/block/stone
freven.vanilla:materials/block/stone
freven.vanilla:models/block/cube_all
example.gems:textures/block/ruby_ore
example.gems:materials/block/ruby_ore
example.gems:models/item/ruby
example.weather:effects/leaves_wind
```

Key rules:

- `namespace` owns the asset identity.
- `path` identifies the asset inside that namespace.
- Keys are author-facing identities, not file paths by themselves.
- Keys should be stable across compatible releases.
- Keys should be lower-case and portable.
- Keys should not contain spaces.
- Keys should not encode renderer slots, atlas coordinates, runtime ids, or cache
  paths.
- The same key must resolve to one effective asset of the expected type after
  the selected experience/mod/content-pack stack is resolved.

The accepted MVP shape follows the existing SDK namespaced resource-key pattern:
`namespace:path`. Future schema documents may narrow the exact character set, but
the long-term rule is stable namespace ownership plus stable path identity.

## Asset categories

Visual asset model v1 recognizes these categories.

| Category | Author-facing role | Typical source |
| --- | --- | --- |
| Texture | Image-like resource used by materials, UI, sprites, particles, or model surfaces | `assets/textures/...` |
| Material | Renderable surface declaration that references textures and render properties | `content/materials/...` |
| Model | Geometry/layout declaration that references material slots or material keys | `content/models/...` or `assets/models/...` depending on format |
| Shader/effect | Named effect or shader declaration with explicit supported inputs/capabilities | `content/effects/...` plus `assets/shaders/...` |
| Atlas/array/load-plan | Generated backend output derived from resolved assets | generated cache |

The model intentionally separates authored declarations from raw files:

- a texture asset may be a file under `assets/`;
- a material is content data that references one or more texture assets;
- a model may be authored content data or an imported asset file with a Freven
  declaration wrapper;
- an atlas or texture array is generated cache, not author-facing source.

## Texture assets

Texture assets are image resources used by materials, UI, particles, sprites,
block visuals, item visuals, model surfaces, or future visual systems.

Texture source examples:

```text
assets/textures/block/stone.png
assets/textures/block/ruby_ore.png
assets/textures/entity/slime_albedo.ktx2
assets/textures/ui/button.png
```

Texture rules:

- authors reference textures by visual asset key, not by renderer slot;
- file paths are package-local source locations, not global identity by
  themselves;
- supported file formats are declared by tooling/runtime policy;
- invalid dimensions, unsupported formats, missing mip data, or failed decoding
  are asset diagnostics;
- texture sampling, color space, compression, mip policy, and GPU upload details
  are not public renderer slots;
- generated atlas coordinates and texture-array layers are host-internal.

Texture authoring, size, sampling, mipmap, alpha, and validation rules are defined by
[TEXTURE_AUTHORING_v1.md](TEXTURE_AUTHORING_v1.md). Packing rules belong to the atlas/array pipeline.

## Material assets

A material is an author-facing surface declaration. It usually references one or
more textures and declares render-facing properties.

Conceptual material example:

```toml
key = "example.gems:materials/block/ruby_ore"
base_color_texture = "example.gems:textures/block/ruby_ore"
fallback_debug_tint_rgba = "C02040FF"
alpha_mode = "opaque"
render_layer = "solid"
```

Material rules:

- materials are content data, not renderer slots;
- materials reference textures by stable asset keys;
- material keys are stable author-facing identities;
- material declarations may include fallback/debug information for incomplete
  renderer support;
- material declarations must not expose palette ids, atlas coordinates, texture
  array layers, GPU handles, or backend pipeline ids;
- Vanilla and standalone experiences provide material libraries through authored
  content/assets, not through engine hardcoding.

The full material schema is defined by [MATERIAL_DEFINITIONS_v1.md](MATERIAL_DEFINITIONS_v1.md).
This model only defines where materials sit in the visual asset graph.

## Model assets

A model is an author-facing geometry or layout declaration. It may be a simple
data-driven shape, an imported model file, or a future animated/entity model
declaration.

Conceptual model example:

```toml
key = "example.gems:models/block/ruby_ore"
kind = "cube"
materials.all = "example.gems:materials/block/ruby_ore"
```

Model rules:

- models reference material keys or named material slots;
- model identity is stable and namespaced;
- geometry import details are not renderer backend handles;
- transforms, origins, UVs, variant selection, and animation hooks belong to
  follow-up model-format documents;
- block, item, entity, and UI visual schemas may all reference model assets.

The full model asset format is a follow-up document. This model only defines
identity and dependency ownership.

## Shader and effect assets

A shader/effect is a named visual behavior declaration. It can reference shader
source files and declare supported inputs, capabilities, and fallback behavior.

Shader/effect rules:

- shader/effect keys are stable author-facing identities;
- shader source files are assets;
- effect declarations are content data;
- renderer backend compilation output is generated cache;
- unsupported effect features must produce diagnostics or fallbacks;
- arbitrary renderer extension is not part of v1.

Shader/effect v1 should be conservative. The platform can expose named effects
and declared inputs before exposing a broad renderer plugin API.

## Generated atlas, array, and load-plan assets

Atlases, texture arrays, compiled assets, and load plans are generated backend
outputs. They are not authored source.

Generated visual outputs may include:

```text
generated atlas pages
texture array layers
material pipeline tables
resolved model/material dependency graph
compiled/transcoded texture data
compiled shader modules
asset fingerprints
validation indexes
```

Generated output rules:

- generated output is rebuildable;
- generated output can be cached;
- generated output can be fingerprinted;
- generated output can be packaged only as an optimization when explicitly marked
  as derived;
- generated output must not replace authored asset identity;
- generated output must not leak as stable author-facing ids.

A material may resolve to a renderer material handle. A texture may resolve to an
atlas layer. A model may resolve to mesh buffers. Those resolved handles are not
the asset keys authors depend on.

## Content data vs asset files

Visual declarations often live in `content/`, while resource files live in
`assets/`.

Example package layout:

```text
mods/example.gems/
  mod.toml
  content/
    materials/
      ruby_ore.toml
    models/
      block_ruby_ore.toml
    visuals/
      blocks.toml
  assets/
    textures/
      block/
        ruby_ore.png
    shaders/
      sparkle.wgsl
```

Ownership:

- `content/materials/ruby_ore.toml` defines a material key and references texture
  keys;
- `content/models/block_ruby_ore.toml` defines a model key and references
  material keys;
- `content/visuals/blocks.toml` maps gameplay/content ids to visual model or
  material keys;
- `assets/textures/block/ruby_ore.png` is image resource data;
- generated atlases and load plans are cache.

This keeps authored meaning separate from resource bytes and generated runtime
outputs.

## Dependency graph

The resolved visual asset graph should be explicit.

Common dependency edges:

```text
block visual -> model
block visual -> material
model -> material slot
model -> material key
material -> texture
material -> effect
effect -> shader source
generated load plan -> resolved texture/material/model/effect graph
```

Rules:

- dependencies must be resolved before runtime use;
- missing keys are diagnostics;
- wrong asset type is a diagnostic;
- cycles are invalid unless a future schema explicitly allows a safe cycle;
- dependency resolution should be deterministic for the selected package stack;
- diagnostics should report the author-facing key and owner package/layer.

## Resolution model

The host resolves visual assets from the selected product, experience, mods, and
content packs.

Conceptual order:

1. Select product/install boundary.
2. Select experience or experience stack.
3. Resolve packages and dependencies.
4. Load visual content declarations and asset files.
5. Apply explicit asset/content layering rules.
6. Validate keys, types, dependencies, and compatibility class.
7. Build a resolved visual asset graph.
8. Build generated cache/load-plan outputs.
9. Hand renderer backend only resolved internal handles.

The result must not depend on accidental filesystem walk order. If two packages
declare the same effective key, the resolver must either apply an explicit
override rule or report a conflict.

Detailed layered override rules are defined in
[LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md).

## Client/server determinism

Not every visual asset is gameplay-authoritative, but resolution still needs a
clear deterministic model.

At minimum tooling should distinguish:

| Class | Meaning |
| --- | --- |
| Server-required visual asset | required by the selected server/experience policy for correct presentation or compatibility |
| Authoritative visual binding | content data binding gameplay ids to model/material/visual keys |
| Client-local cosmetic asset | allowed local visual replacement that does not alter authoritative gameplay identity |
| Generated visual cache | rebuildable host/devkit output |

Rules:

- authoritative visual bindings participate in compatibility identity;
- server-required assets may participate in server-required fingerprints;
- client-local cosmetic overrides must not change authoritative gameplay meaning;
- generated cache is not durable truth;
- the same selected stack should produce the same resolved graph before backend
  cache generation.

## Validation model

Validation should catch at least:

- invalid `namespace:path` key shape;
- duplicate effective key without explicit override;
- missing texture/material/model/effect key;
- wrong asset type for a reference;
- invalid package-relative path;
- unsupported image/model/shader format;
- invalid or unsupported color space/compression/sampling metadata;
- invalid dependency cycle;
- material references missing texture inputs;
- model references missing material slots;
- shader/effect references unsupported runtime capability;
- renderer-internal ids in authored content;
- server-required asset missing on client;
- client-local cosmetic override used where server policy forbids it.

Diagnostics should name the author-facing key, category, owner package, selected
layer, and the file that should be fixed.

## Relationship to existing debug-palette block visuals

The current block descriptor helpers include debug-colored blocks and namespaced
material keys as a transition path.

Rules:

- explicit `material_id` values are legacy/debug-palette slots;
- normal authors should not guess palette slots;
- namespaced material keys are the forward-compatible bridge;
- fallback debug tint keeps content visible before real material resolution;
- host/runtime still resolves authored keys into renderer-internal palette,
  atlas, texture-array, or backend handles.

This document defines the long-term visual asset model that the material-key path
is preparing for.

## Vanilla and standalone boundaries

Vanilla may provide first-party textures, materials, models, effects, and visual
bindings. It does so as authored content/assets.

The engine must not hardcode Vanilla visual style. A zero-Vanilla standalone
game should be able to provide its own complete visual asset library with the
same model.

Examples:

```text
freven.vanilla:textures/block/stone
freven.vanilla:materials/block/stone
example.space:textures/hull/panel
example.space:materials/hull/panel
example.space:models/entity/drone
```

The engine owns loading, validation, resolution, caching, and rendering. The
experience owns authored visual meaning.

## DevKit guidance

DevKit commands should eventually be able to explain:

- which package owns a visual asset key;
- which file declared it;
- which asset files it depends on;
- whether it is authoritative, server-required, cosmetic, or generated;
- whether an override is explicit and valid;
- why a texture/material/model/effect failed validation;
- which generated cache entries can be rebuilt.

Diagnostics should avoid telling authors to edit generated cache or renderer
backend handles.

## Follow-up documents and issues

This model is intentionally foundational.

Follow-up work should define:

- material schema v1, defined by [MATERIAL_DEFINITIONS_v1.md](MATERIAL_DEFINITIONS_v1.md);
- texture metadata and authoring policy, defined by [TEXTURE_AUTHORING_v1.md](TEXTURE_AUTHORING_v1.md);
- atlas/array pipeline;
- block visual data schema;
- model asset format v1;
- shader/effect extension boundary;
- layered asset override rules defined by [LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md);
- content patch/merge semantics defined by [CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md);
- DevKit validation commands and generated load-plan output.

Those documents should reference this visual asset model instead of redefining
visual asset identity from scratch.
