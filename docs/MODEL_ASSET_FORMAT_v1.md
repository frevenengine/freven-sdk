# Model Asset Format v1

This document defines the Freven rc10 model asset format contract.

It builds on:

- [ARCHITECTURE.md](ARCHITECTURE.md): engine / SDK / experience / mod /
  content-pack / standalone-product ownership;
- [PACKAGE_BOUNDARIES.md](PACKAGE_BOUNDARIES.md): manifest, config, content,
  assets, generated cache, and save/world state ownership;
- [VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md): stable visual asset
  identity, model asset category, dependency graph, validation, and
  renderer-internal boundaries;
- [MATERIAL_DEFINITIONS_v1.md](MATERIAL_DEFINITIONS_v1.md): data-driven
  material declarations referenced by models and visuals;
- [TEXTURE_AUTHORING_v1.md](TEXTURE_AUTHORING_v1.md): texture size, sampling,
  mipmap, alpha, color-space, and validation policy;
- [TEXTURE_BACKEND_PIPELINE_v1.md](TEXTURE_BACKEND_PIPELINE_v1.md): generated
  atlas/texture-array/backend planning and renderer-internal slot boundaries;
- [BLOCK_VISUAL_DEFINITIONS_v1.md](BLOCK_VISUAL_DEFINITIONS_v1.md): block visual
  bindings from gameplay block keys to model/material/tint/render policy;
- [LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md): deterministic
  visual asset layering and override policy;
- [CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md): semantic add,
  replace, patch, append, disable, compatibility, and diagnostics model;
- [CREATOR_CONTENT_SCHEMA_v1.md](CREATOR_CONTENT_SCHEMA_v1.md): creator-facing
  source schema direction;
- [DATA_DRIVEN_AUTHORING_LAYER_v1.md](DATA_DRIVEN_AUTHORING_LAYER_v1.md):
  practical authoring workflow and shorthand expansion.

The goal is to define a stable author-facing model format for block, item, and
future entity/static visuals without exposing renderer meshes, atlas coordinates,
GPU buffers, Bevy/wgpu handles, runtime block ids, or generated cache paths.

## Core rule

Models are visual asset declarations.

A model describes reusable geometry, layout, material slots, UV intent,
transforms, and future animation/import hooks. A model is not a renderer mesh,
not a GPU buffer, not an atlas region, not a runtime entity, not a collision
shape, and not generated cache.

Author-facing model:

~~~toml
schema = 1
key = "example.gems:models/block/ruby_ore"

[model]
kind = "cube"

[material_slots]
all = { required = true, profile = "voxel_block" }
~~~

Host/backend output:

~~~text
chunk mesh vertices
renderer material slot
atlas page and rectangle
texture array layer
GPU vertex/index buffer
Bevy mesh handle
wgpu buffer/bind group
generated mesh/cache artifact
~~~

Only the first model is public SDK vocabulary.

## Goals

- Define stable `namespace:path` model keys.
- Support simple block cube models.
- Support per-face cube models.
- Support custom block models made of cuboid parts.
- Support item models through the same author-facing model identity.
- Reserve future-compatible entity/static mesh and imported asset wrappers.
- Define model-local material slots separately from resolved material keys.
- Define block-local coordinates, origins, transforms, rotations, and UV intent.
- Keep collision and selection shapes separate from visual model geometry.
- Keep renderer-internal mesh buffers, slots, atlas coordinates, and GPU handles
  out of authored data.
- Provide a clean SDK target for engine block model meshing v1.

## Non-goals

This document does not define:

- final engine meshing implementation;
- final GPU vertex layout;
- Bevy/wgpu resource layout;
- full glTF/import pipeline;
- skeletal animation or animation controller format;
- final entity renderer;
- final item renderer;
- final collision or selection schema;
- final variant/family expansion schema;
- Vanilla's actual model library.

Those are separate rc10 SDK, engine, DevKit, and Vanilla issues.

## Terminology

| Term | Meaning |
| --- | --- |
| Model asset | Author-facing visual asset declaration with a stable key. |
| Model key | Stable namespaced model identity such as `example:models/block/crate`. |
| Model kind | High-level model shape such as `cube`, `cube_faces`, `cuboid_parts`, or `imported_static`. |
| Part | A named cuboid or future imported submesh inside a model. |
| Face | One side of a cuboid: top, bottom, north, south, east, or west. |
| Material slot | Author-facing model-local slot name such as `all`, `top`, `frame`, or `pane`. |
| Material binding | A block/item/entity visual binding from a slot name to a material key. |
| UV | Author-facing texture coordinate intent for a face or surface. |
| Origin | Local pivot/reference point for transforms and rotations. |
| Transform | Author-facing translation/rotation/scale intent. |
| Renderer mesh | Host/backend generated vertex/index data. Not authored source. |
| Imported asset | External asset file such as glTF wrapped by a Freven model declaration. |

## Stable model keys

A model key is a stable visual asset key:

~~~text
namespace:models/<domain>/<name>
~~~

Examples:

~~~text
freven.core:models/block/cube_all
freven.core:models/block/cube_faces
freven.vanilla:models/block/glass_pane
example.gems:models/block/ruby_ore
example.gems:models/item/ruby
example.creatures:models/entity/firefly
example.props:models/static/crate
~~~

Rules:

- the namespace owns the model identity;
- the path identifies the model inside that namespace;
- model keys should be stable across compatible releases;
- model keys must not encode renderer slots, runtime block ids, entity ids, atlas
  coordinates, texture-array layers, GPU handles, or cache paths;
- the selected visual asset graph must resolve each effective model key to one
  model declaration;
- replacing or patching a model uses content patch/merge semantics.

## Ownership and location

Model declarations live in content data, normally under:

~~~text
content/models/
~~~

External imported model source files, when used, live in assets, normally under:

~~~text
assets/models/
~~~

Generated mesh caches, compiled buffers, optimized meshlets, atlas bindings, and
renderer-specific outputs live in generated cache and are rebuildable.

Example package layout:

~~~text
mods/example.gems/
  mod.toml
  content/
    models/
      block/
        ruby_ore.toml
      item/
        ruby.toml
  assets/
    models/
      entity/
        firefly.glb
    textures/
      block/
        ruby_ore.png
~~~

## Model schema v1

Canonical model files use:

~~~toml
schema = 1
key = "example.gems:models/block/ruby_ore"

[model]
kind = "cube"
domain = "block"

[material_slots]
all = { required = true, profile = "voxel_block" }
~~~

### Required fields

| Field | Type | Meaning |
| --- | --- | --- |
| `schema` | integer | Model file schema version. v1 uses `1`. |
| `key` | `namespace:path` | Stable model identity. |
| `model.kind` | enum | Model kind. |

### Common fields

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `model.domain` | enum | inferred from key path when possible | Intended use: `block`, `item`, `entity`, `static`, `ui`. |
| `model.bounds` | object | derived from parts | Author-facing local visual bounds. |
| `model.origin` | vec3 | `[0.5, 0.5, 0.5]` for block models | Local pivot/reference point. |
| `model.transform` | object | identity | Optional model-level transform. |
| `model.cull_policy` | enum | kind-dependent | Face/part culling hint. |
| `model.ambient_occlusion` | bool | true for voxel/block models | AO participation hint. |
| `model.tint_slots` | list/map | none | Slots or faces that accept visual tint. |
| `material_slots` | map | kind-dependent | Model-local material slot declarations. |
| `parts` | array | kind-dependent | Cuboid parts for `cuboid_parts`. |
| `import` | object | none | Future imported asset wrapper. |
| `animation_hooks` | map/list | none | Future animation hook declarations. |

The schema records author intent. Renderer-specific output is generated by the
host/runtime.

## Model kinds

Model format v1 recognizes these kind names.

| Kind | Meaning |
| --- | --- |
| `cube` | One full block cube with a single material slot. |
| `cube_faces` | One full block cube with per-face and shorthand material slots. |
| `cuboid_parts` | One or more axis-aligned cuboid parts with per-face slots and UVs. |
| `item_flat` | Simple item/icon-oriented model wrapper. |
| `item_model` | Item model that reuses block/cuboid/import model semantics. |
| `imported_static` | Future-compatible wrapper around imported static mesh assets. |
| `entity_static` | Future-compatible entity/static visual declaration. |

Rules:

- `cube`, `cube_faces`, and `cuboid_parts` are the rc10 block model v1 core;
- item/entity/imported kinds may be declared as future-compatible contracts even
  if the current renderer handles only a subset;
- unsupported model kinds must produce diagnostics or deterministic fallback;
- kind names are author-facing schema vocabulary, not renderer pipeline ids.

## Coordinate system

Block-style model coordinates are local and normalized.

Recommended rc10 canonical coordinate space:

~~~text
[0.0, 0.0, 0.0] = minimum local block corner
[1.0, 1.0, 1.0] = maximum local block corner
~~~

Axis convention:

| Axis | Positive direction |
| --- | --- |
| X | east |
| Y | top/up |
| Z | south |

Face convention:

| Face | Normal |
| --- | --- |
| `top` | +Y |
| `bottom` | -Y |
| `north` | -Z |
| `south` | +Z |
| `east` | +X |
| `west` | -X |

Rules:

- authored coordinates are stable content values;
- renderer coordinate transforms are backend details;
- tools may offer pixel-grid aliases such as 0..16, but canonical semantic
  values should compile to normalized coordinates;
- coordinates outside `[0.0, 1.0]` are allowed only if the selected model policy
  explicitly supports oversized visuals;
- oversized visuals must not silently expand gameplay collision or selection.

## Material slots

A material slot is a model-local author-facing name.

A model declares slots. A block/item/entity visual binds those slots to material
keys.

Model declaration:

~~~toml
schema = 1
key = "example.glass:models/block/framed_glass"

[model]
kind = "cuboid_parts"
domain = "block"

[material_slots]
frame = { required = true, profile = "voxel_block" }
pane = { required = true, profile = "voxel_block" }
~~~

Block visual binding:

~~~toml
schema = 1
key = "example.glass:visuals/block/framed_glass"

[visual]
target = "example.glass:blocks/framed_glass"
model = "example.glass:models/block/framed_glass"

[visual.materials]
frame = "example.glass:materials/block/iron_frame"
pane = "example.glass:materials/block/blue_glass"
~~~

Rules:

- model files declare slot names and where slots are used;
- visual binding files provide material keys for those slots;
- material slots are not renderer material slots;
- missing required slot bindings are validation errors;
- unknown visual material slots are validation errors unless the model explicitly
  declares extension slots;
- the same material key may bind to many model slots;
- one model may be reused by many visuals with different material bindings.

## Cube model

A `cube` model represents one full local block cube.

~~~toml
schema = 1
key = "freven.core:models/block/cube_all"

[model]
kind = "cube"
domain = "block"
origin = [0.5, 0.5, 0.5]

[material_slots]
all = { required = true, profile = "voxel_block" }

[cube]
from = [0.0, 0.0, 0.0]
to = [1.0, 1.0, 1.0]
material = "all"
~~~

Rules:

- `cube` has one canonical required material slot: `all`;
- all six faces use the `all` slot unless a future extension says otherwise;
- full-block cube geometry does not imply full-block collision;
- face culling may be inferred by the engine but authored culling hints remain
  semantic hints, not renderer state.

## Per-face cube model

A `cube_faces` model represents one full cube with face-specific slots.

~~~toml
schema = 1
key = "freven.core:models/block/cube_faces"

[model]
kind = "cube_faces"
domain = "block"
origin = [0.5, 0.5, 0.5]

[material_slots]
top = { required = false, profile = "voxel_block" }
bottom = { required = false, profile = "voxel_block" }
north = { required = false, profile = "voxel_block" }
south = { required = false, profile = "voxel_block" }
east = { required = false, profile = "voxel_block" }
west = { required = false, profile = "voxel_block" }
side = { required = false, profile = "voxel_block" }
all = { required = false, profile = "voxel_block" }
~~~

Resolution rules for cube-style slots:

1. A face-specific slot wins.
2. For north/south/east/west, `side` may provide a fallback.
3. `all` may provide a fallback for any missing face.
4. If a rendered face has no resolved material slot, validation fails unless the
   model declares a fallback policy.

This keeps grass/dirt/log/machine-style visuals compact without inventing
renderer-specific per-face ids.

## Cuboid parts model

A `cuboid_parts` model contains one or more axis-aligned cuboids.

~~~toml
schema = 1
key = "example.glass:models/block/framed_glass"

[model]
kind = "cuboid_parts"
domain = "block"
origin = [0.5, 0.5, 0.5]
ambient_occlusion = true

[material_slots]
frame = { required = true, profile = "voxel_block" }
pane = { required = true, profile = "voxel_block" }

[[parts]]
name = "center_pane"
from = [0.4375, 0.0, 0.4375]
to = [0.5625, 1.0, 0.5625]

[parts.faces.north]
material = "pane"
uv = [0.0, 0.0, 1.0, 1.0]

[parts.faces.south]
material = "pane"
uv = [0.0, 0.0, 1.0, 1.0]

[[parts]]
name = "top_frame"
from = [0.0, 0.875, 0.0]
to = [1.0, 1.0, 1.0]
material = "frame"
~~~

Part fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `name` | string | Stable model-local part name. |
| `from` | vec3 | Minimum local coordinate. |
| `to` | vec3 | Maximum local coordinate. |
| `material` | slot name | Default material slot for all faces of the part. |
| `faces` | map | Per-face overrides. |
| `rotation` | object | Optional author-facing local rotation. |
| `origin` | vec3 | Optional part-local rotation origin. |
| `cull` | bool/enum | Culling hint. |
| `tint` | string/bool | Tint participation hint. |

Rules:

- v1 block meshable geometry is cuboid-based;
- cuboids must have positive size on all axes;
- parts should have stable names for diagnostics and patching;
- part order must be deterministic;
- overlapping parts are allowed but may produce overdraw warnings;
- non-axis-aligned final geometry may be represented through part rotation where
  supported, or deferred to future imported/static model support.

## Faces and UVs

A part face may override material, UV, culling, tint, and rotation.

~~~toml
[parts.faces.top]
material = "top"
uv = [0.0, 0.0, 1.0, 1.0]
rotation = 0
cull = true
tint = "biome_grass"
~~~

Face fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `material` | slot name | Model-local material slot. |
| `uv` | vec4 | Normalized UV rectangle `[u0, v0, u1, v1]`. |
| `rotation` | enum/int | UV rotation, normally `0`, `90`, `180`, or `270`. |
| `cull` | bool/enum | Face culling hint. |
| `tint` | string/bool | Tint participation hint. |

Rules:

- UVs are author-facing texture-coordinate intent;
- atlas coordinates and texture-array layers must never appear in model files;
- if UVs are omitted, default face UVs are generated deterministically;
- UV rotation is semantic, not renderer state;
- invalid UV rectangles are diagnostics;
- material slot names must exist in `material_slots`.

## Transforms, origins, and rotations

Model v1 supports author-facing transforms as semantic layout data.

Conceptual model-level transform:

~~~toml
[model.transform]
translation = [0.0, 0.0, 0.0]
rotation_degrees = [0.0, 0.0, 0.0]
scale = [1.0, 1.0, 1.0]
~~~

Conceptual part rotation:

~~~toml
[parts.rotation]
axis = "y"
degrees = 45.0
origin = [0.5, 0.5, 0.5]
~~~

Rules:

- transforms are authored semantic data;
- renderer matrices, shader constants, and GPU buffers are backend details;
- rotations should be restricted by model kind and renderer capability;
- block model v1 may support only axis-aligned cuboids at first and diagnose
  unsupported rotations;
- unsupported transforms must fall back deterministically or fail validation;
- transforms must not change collision or selection unless gameplay content
  explicitly declares matching changes.

## Item models

Item models can reuse the same model asset vocabulary.

Simple item model:

~~~toml
schema = 1
key = "example.gems:models/item/ruby"

[model]
kind = "item_flat"
domain = "item"

[material_slots]
icon = { required = true, profile = "voxel_item" }

[item]
display_transform = "generated_icon"
~~~

Block-as-item model:

~~~toml
schema = 1
key = "example.gems:models/item/ruby_block"

[model]
kind = "item_model"
domain = "item"
source_model = "example.gems:models/block/ruby_block"
~~~

Rules:

- item models use stable model keys;
- item renderer-specific camera transforms are author-facing display hints, not
  GPU state;
- item visuals bind material slots through item visual/content bindings;
- full item rendering implementation can lag behind the model format contract.

## Entity and static models

Model v1 reserves future-compatible entity/static model declarations.

Conceptual imported static wrapper:

~~~toml
schema = 1
key = "example.creatures:models/entity/firefly"

[model]
kind = "imported_static"
domain = "entity"

[import]
source = "example.creatures:assets/models/entity/firefly.glb"
format = "gltf"
scale = [1.0, 1.0, 1.0]

[material_slots]
body = { required = true, profile = "model_surface" }
wing = { required = false, profile = "model_surface" }
~~~

Rules:

- imported source files live in `assets/`;
- the model declaration wraps the imported asset with stable Freven semantics;
- imported file node/material names are not stable public Freven identity unless
  explicitly mapped to material slots;
- skeletal animation, skinning, animation graphs, and entity renderer behavior are
  future work;
- unsupported imported model kinds must produce diagnostics or deterministic
  fallback.

## Animation hooks

Model v1 may reserve named animation hooks without defining the full animation
system.

~~~toml
[animation_hooks]
open = { kind = "transform", target = "lid" }
idle = { kind = "external", target = "example.creatures:animations/firefly_idle" }
~~~

Rules:

- hooks are stable names consumed by future animation/effect systems;
- hooks do not execute behavior by themselves;
- missing animation providers are diagnostics where the selected renderer requires
  them;
- gameplay state and behavior remain separate from model declaration.

## Variant hooks

Model v1 can expose variant-facing slots and transform hooks, but it does not
define family expansion.

~~~toml
[variant_hooks]
axis = ["wood_species"]
affects = ["material_slots", "parts", "transform"]
~~~

Rules:

- variant hooks are declarations of intended variation points;
- generated model keys and generated entries are owned by the variant/family
  expansion schema;
- runtime ids remain internal;
- diagnostics should show both source model key and generated effective model
  keys when expansion is used.

## Collision and selection are not model geometry

Visual model geometry is not gameplay collision.

Rules:

- model parts affect rendering;
- collision shapes affect physics/gameplay;
- selection shapes affect raycast/highlight/tool interaction;
- changing model geometry must not silently change collision or selection;
- client-local cosmetic packs may not change collision or selection by replacing
  a model;
- DevKit should warn when a model visually extends far beyond declared collision
  or selection policy.

The final collision and selection schema is owned by gameplay/block content work.

## Composition and patching

Models compose through content patch/merge semantics.

Examples:

~~~toml
[[content_patches]]
op = "replace"
kind = "model"
target = "freven.vanilla:models/block/glass"
replacement = "example.pack:models/block/clear_glass"
authority = "selected_stack"
reason = "selected visual refresh"
~~~

~~~toml
[[content_patches]]
op = "patch"
kind = "model"
target = "example.glass:models/block/framed_glass"
path = "parts.top_frame.to"
value = [1.0, 0.95, 1.0]
authority = "selected_stack"
reason = "thin frame variant"
~~~

Rules:

- adding, replacing, or patching models must be explicit;
- accidental duplicate model keys are conflicts;
- patchable fields must be declared by schema/tooling policy;
- patches must not write renderer mesh ids, GPU handles, atlas coordinates,
  texture-array layers, runtime block ids, or generated cache paths;
- model patches must not modify gameplay collision, selection, drops, providers,
  save/world state, or runtime behavior.

## Authority and compatibility

Models may be cosmetic, server-required, or authoritative depending on selected
product/experience/server policy.

| Class | Meaning |
| --- | --- |
| Cosmetic model | Local presentation-only model replacement allowed by policy. |
| Server-required model | Required effective model key/hash or accepted equivalent. |
| Authoritative model binding | Model reference that participates in selected stack identity. |
| Generated model cache | Rebuildable host/devkit output derived from effective model graph. |

Rules:

- a model asset by itself is visual content;
- a visual binding may make a model server-required or authoritative;
- changing a model cannot change gameplay meaning unless explicit gameplay
  content changes also exist;
- required model declarations may participate in compatibility fingerprints;
- diagnostics should explain the model's effective authority class.

## Validation

Model validation should happen before runtime start.

Validation should catch:

- missing `schema`;
- unsupported schema version;
- missing or invalid model key;
- invalid `model.kind`;
- invalid or unsupported `model.domain`;
- invalid coordinate values;
- cuboid `from` greater than or equal to `to`;
- unknown face name;
- duplicate part name;
- missing required material slot declaration;
- face or part references an undeclared material slot;
- visual binding provides unknown material slot for a model;
- invalid UV rectangle;
- unsupported transform or rotation;
- unsupported imported asset format;
- missing imported asset source;
- imported asset node/material mapping ambiguity;
- duplicate model key;
- renderer-internal id in authored data;
- runtime block/entity id in authored data;
- atlas coordinate or texture-array layer in authored data;
- generated cache referenced as authored source;
- model patch attempting to modify collision, selection, or gameplay fields.

Diagnostics should report:

- file path;
- line/field path where possible;
- model key;
- model kind;
- part name;
- face name;
- material slot name;
- referenced material/model/asset key;
- owner package/layer;
- selected override/patch operation;
- expected type or allowed values;
- suggested fix.

## Relationship to block visuals

[BLOCK_VISUAL_DEFINITIONS_v1.md](BLOCK_VISUAL_DEFINITIONS_v1.md) defines how a
gameplay block key chooses a model key and binds material slots.

This document defines what the model key points to.

If there is disagreement:

- block visuals own block-to-model binding;
- model format owns geometry/layout/material-slot declarations;
- material definitions own surface properties;
- texture policy owns texture source validation;
- engine meshing owns generated renderer mesh output.

## Relationship to texture backend

Model files may declare material slots and UV intent, but they do not declare
atlas coordinates, texture-array layers, GPU samplers, or renderer slot ids.

Texture backend planning consumes resolved materials and textures after content
composition. It may generate atlas pages, texture arrays, cache records, material
tables, and GPU resources. Those outputs are host/backend state, not model
source.

## Relationship to variants/families

This document reserves variant hooks and defines fields that generated models may
use.

The variant/family expansion schema defines:

- variant axes;
- generated model keys;
- generated block/material/model/visual entries;
- skip/allow lists;
- per-variant overrides;
- diagnostics for invalid combinations.

If there is disagreement, family expansion owns generated entry creation, while
this document owns the shape and validation of each resulting model entry.

## Relationship to engine meshing

The engine block model mesher should consume effective, validated model and block
visual data.

Engine meshing owns:

- conversion from model declarations to chunk mesh vertices;
- renderer-internal material slot lookup;
- render-layer grouping;
- tint-aware mesh output;
- culling and AO implementation details;
- generated mesh/cache artifacts;
- performance counters and renderer diagnostics.

The SDK model format owns author-facing shape, identity, dependencies, and
validation vocabulary.

## DevKit guidance

DevKit should eventually be able to:

- validate model files;
- show model key, kind, domain, parts, faces, and material slots;
- show resolved visual bindings that use a model;
- explain missing or unknown material slots;
- preview cuboid parts and per-face slots;
- show UV rectangles and default-generated UVs;
- report unsupported transforms/imports before runtime start;
- show whether a model is cosmetic, selected-stack, server-required, or
  authoritative;
- show renderer-internal mesh/atlas/slot data only in inspector/debug views,
  never as authoring fields;
- generate starter models for cube, per-face cube, framed glass, pane, slab,
  item icon, and imported static wrappers.

## Summary

Model asset format v1 defines stable author-facing model declarations for
Freven visual content.

Models own reusable geometry, layout, material slots, UV intent, transforms, and
future import/animation hooks while keeping gameplay behavior, collision,
selection, runtime ids, renderer slots, atlas coordinates, GPU handles, and
generated cache out of authored model data.
