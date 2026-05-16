# Texture Backend Pipeline v1

This document defines the Freven rc10 SDK contract for turning validated texture
assets and material texture references into generated renderer-backend texture
plans.

It builds on:

- [ARCHITECTURE.md](ARCHITECTURE.md): engine / SDK / experience / mod /
  content-pack / standalone-product ownership;
- [PACKAGE_BOUNDARIES.md](PACKAGE_BOUNDARIES.md): assets, content data,
  generated cache, and save/world state boundaries;
- [VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md): stable visual asset keys
  and renderer-internal backend handles;
- [MATERIAL_DEFINITIONS_v1.md](MATERIAL_DEFINITIONS_v1.md): material texture
  references and material-facing texture inputs;
- [TEXTURE_AUTHORING_v1.md](TEXTURE_AUTHORING_v1.md): texture size, format,
  sampling, mipmap, alpha, color-space, and validation policy;
- [LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md): effective
  texture override selection and asset graph fingerprints;
- [SHADER_EFFECT_BOUNDARY_v1.md](SHADER_EFFECT_BOUNDARY_v1.md): shader/effect
  ownership boundary for effect texture capabilities and generated shader/cache
  boundaries.

## Goals

- Define the generated texture backend plan as host/runtime output, not authored
  source.
- Keep stable author-facing texture and material keys separate from internal
  atlas regions, texture-array layers, GPU handles, Bevy handles, wgpu handles,
  and cache paths.
- Define deterministic grouping and ordering for resolved texture assets.
- Define strategy vocabulary for texture arrays, atlases, and hybrid plans.
- Define duplicate detection, hashing, cache invalidation, and diagnostics.
- Define how material texture references bind to internal renderer texture
  entries without exposing those entries as SDK API.
- Give engine issue #278 a stable SDK-side contract to target.

## Non-goals

This document does not define:

- concrete renderer implementation;
- Bevy or wgpu resource layout;
- shader module layout;
- bind group layout;
- exact GPU texture formats;
- final atlas packing algorithm implementation;
- final texture memory budget implementation;
- Vanilla's actual texture files;
- marketplace/resource-pack UI.

Those belong to engine, DevKit, Vanilla, and product work.

## Core rule

Authors reference stable keys. The host generates backend handles.

Author-facing inputs:

~~~text
example.gems:textures/block/ruby_ore
example.gems:materials/block/ruby_ore
assets/textures/block/ruby_ore.png
~~~

Generated backend outputs:

~~~text
texture array layer
atlas page and rectangle
internal texture table index
material texture binding slot
GPU texture/view/sampler
Bevy Handle<Image>
generated cache artifact
~~~

The generated backend outputs must not become public authored values.

## Pipeline overview

The conceptual pipeline is:

~~~text
selected package stack
  -> effective visual asset graph
  -> validated texture assets
  -> material texture dependency graph
  -> texture backend grouping
  -> deterministic backend plan
  -> generated cache artifacts
  -> renderer-internal texture/material handles
~~~

The SDK contract describes the vocabulary and invariants. The engine decides the
concrete renderer implementation.

## Inputs

The texture backend pipeline consumes only resolved, validated inputs:

| Input | Source |
| --- | --- |
| Effective texture key | visual asset resolver |
| Effective texture source bytes | selected package/assets layer |
| Texture profile | texture authoring policy or context inference |
| Dimensions | decoded/validated image metadata |
| Source format | texture authoring policy |
| Color-space/data-kind | material field and texture authoring policy |
| Sampling profile | material or texture policy |
| Mipmap policy | sampling profile and texture policy |
| Alpha/material usage | material definitions |
| Effective material texture references | material definitions |
| Override winner | layered asset resolver |
| Source asset hash | asset validation |

The pipeline must not consume filesystem walk order, hash-map iteration order,
renderer object ids, or previous cache slot ids as semantic input.

## Outputs

The texture backend pipeline may produce:

| Output | Visibility |
| --- | --- |
| Texture backend plan | host/devkit generated cache |
| Texture group records | host/devkit generated cache |
| Material texture binding records | host/devkit generated cache |
| Atlas pages | renderer/internal generated cache |
| Texture arrays | renderer/internal generated cache |
| Compiled/transcoded images | renderer/internal generated cache |
| Cache fingerprints | host/devkit generated cache |
| Diagnostics | user-facing |
| GPU handles | renderer-internal only |

Only diagnostics and stable author-facing keys are public author surfaces.

## Strategy vocabulary

A backend plan may use one or more strategies.

| Strategy | Meaning |
| --- | --- |
| `texture_array_2d` | Store compatible same-size 2D textures as array layers. |
| `atlas_2d` | Pack compatible 2D textures into atlas pages and rectangles. |
| `hybrid` | Use arrays for strict voxel groups and atlases for UI/effects/model groups. |
| `single_texture` | Keep a texture as an individual backend resource when packing is unsuitable. |
| `fallback_debug` | Use fallback color/material path for missing or unsupported texture inputs. |

Rules:

- strategy names are diagnostic/plan vocabulary, not author-facing slots;
- an experience/product may choose default strategies by profile;
- engine capabilities may restrict allowed strategies;
- fallback strategy must be visible in diagnostics;
- the selected strategy participates in generated cache fingerprints.

Recommended rc10 defaults:

| Texture profile | Recommended strategy |
| --- | --- |
| `voxel_block` | `texture_array_2d` |
| `voxel_item` | `texture_array_2d` or `atlas_2d` |
| `model_surface` | `single_texture` or `texture_array_2d` by material set |
| `ui` | `atlas_2d` or `single_texture` |
| `effect_sprite` | `atlas_2d` |

## Grouping

Textures can share one backend group only when compatible.

Required grouping dimensions:

| Group key part | Why |
| --- | --- |
| Texture profile | Voxel/UI/model/effect policy differs. |
| Width and height | Texture arrays require compatible dimensions. |
| Source/decoded format class | Backend upload compatibility. |
| Color-space/data-kind | sRGB color and linear/data textures must not be mixed incorrectly. |
| Sampling profile | Sampler/mip behavior must match or be represented explicitly. |
| Mipmap policy | Generated/provided/none changes backend output. |
| Alpha usage class | Opaque/cutout/blend buckets may require different renderer paths. |
| Strategy | Atlas and array records are not interchangeable. |
| Product/renderer capability class | Backends may support different limits. |

For texture arrays, width and height normally must match inside a group.

For atlases, dimensions may vary inside a group, but format, color-space,
sampling, mipmap, and alpha policy still need compatible handling.

## Deterministic ordering

The same selected package stack and source bytes must produce the same plan.

Required ordering rules:

1. Resolve effective texture assets before backend planning.
2. Sort backend candidates by effective texture key.
3. Use stable tie-breakers only from authored/effective metadata.
4. Never use filesystem traversal order.
5. Never use hash-map iteration order.
6. Never use previous cache order as semantic input.
7. Emit diagnostics in stable key order.
8. Emit generated plan records in stable key order within each group.

Suggested candidate sort key:

~~~text
texture_key
profile
width
height
color_space_or_data_kind
sampling_profile
mip_policy
source_asset_hash
~~~

The source hash is a tie-breaker, not an identity replacement for the key.

## Duplicate detection

There are several duplicate-like cases.

| Case | Behavior |
| --- | --- |
| Same key declared twice in one layer | duplicate-key diagnostic before backend planning. |
| Same key from multiple layers | resolved by layered asset override policy before backend planning. |
| Different keys with identical bytes | may dedupe internally, but keys remain distinct author identities. |
| Different keys pointing to same package-local file | allowed or warned by tooling policy. |
| Same material references same texture multiple times | material binding may reuse one internal texture entry. |
| Same texture used by many materials | backend plan should reuse one resolved texture entry. |

Rules:

- byte dedupe must never erase author-facing keys from diagnostics;
- cache records may store one artifact for identical bytes;
- material binding records should still mention the material field and texture key;
- duplicate detection must be deterministic.

## Hashes and fingerprints

The pipeline uses hashes for validation, dedupe, and cache invalidation.

A texture backend input fingerprint should include:

- effective texture key;
- source asset bytes hash;
- selected package/layer identity;
- source format;
- decoded dimensions;
- texture profile;
- color-space/data-kind;
- sampling profile;
- mipmap policy;
- alpha usage class;
- selected backend strategy;
- packing policy version;
- padding/extrusion policy version;
- relevant engine/product capability version.

It should not include:

- absolute install path;
- filesystem discovery order;
- generated atlas coordinate;
- generated texture-array layer;
- renderer slot id;
- GPU handle;
- Bevy/wgpu handle;
- generated cache path as durable truth.

Generated cache fingerprints are useful for invalidation. They are not authored
source and are not durable save/world state.

## Cache invalidation

Generated texture backend cache must be invalidated when any fingerprint input
changes.

Invalidate or rebuild when:

- source bytes change;
- texture dimensions change;
- texture profile changes;
- sampling or mipmap policy changes;
- color-space/data-kind changes;
- alpha usage class changes;
- material texture reference changes;
- override winner changes;
- packing policy changes;
- padding/extrusion policy changes;
- backend strategy changes;
- engine/product capability version changes.

Do not invalidate because of:

- absolute install path moving while package identity and source hash stay stable;
- cache file timestamp alone;
- previous internal slot id changing;
- GPU handle identity changing.

## Atlas policy

An atlas is generated backend output.

Atlas plan records may include internal data such as:

~~~text
atlas_page_id
x
y
width
height
padding
extrusion
mip_level_count
source_texture_key
~~~

Rules:

- atlas page ids and rectangles are internal;
- atlas coordinates must not appear in authored material, texture, block, item,
  model, or visual binding files;
- atlas records may be shown in diagnostics for debugging, but only as generated
  backend details;
- atlas packing order must be deterministic;
- atlas overflow must produce diagnostics;
- stale atlas cache must be invalidated/rebuilt.

Padding/extrusion policy is required when mipmaps, filtering, or UV edges can
sample neighboring atlas entries.

## Texture array policy

A texture array is generated backend output.

Texture-array plan records may include internal data such as:

~~~text
array_id
layer_index
width
height
mip_level_count
source_texture_key
~~~

Rules:

- array ids and layer indexes are internal;
- texture-array layers must not appear in authored material, texture, block, item,
  model, or visual binding files;
- array layer order must be deterministic;
- array groups normally require matching dimensions and compatible format policy;
- layer limit overflow must produce diagnostics;
- stale texture-array cache must be invalidated/rebuilt.

Texture arrays are the recommended rc10 default for strict same-size voxel block
texture groups.

## Hybrid plans

A hybrid plan may combine strategies.

Example:

~~~text
voxel_block 32x32 sRGB nearest -> texture_array_2d
voxel_block 32x32 data nearest -> texture_array_2d
ui sRGB linear                 -> atlas_2d
model_surface sRGB linear_mip  -> single_texture
~~~

Rules:

- each group has one selected strategy;
- material texture bindings point to internal group entries;
- diagnostics must explain why a group used atlas, array, single texture, or
  fallback;
- hybrid strategy must not leak internal ids into authored content.

## Material texture bindings

Materials reference texture keys.

Backend plans resolve material texture fields into internal texture entries:

| Material field | Backend binding |
| --- | --- |
| `base_color_texture` | internal color texture entry |
| `normal_texture` | internal data texture entry |
| `metallic_roughness_texture` | internal data texture entry |
| `emissive_texture` | internal color texture entry |
| `occlusion_texture` | internal data texture entry |

A generated binding record may include:

~~~text
material_key
material_field
texture_key
backend_group_id
internal_texture_entry_id
fallback_state
diagnostics
~~~

Rules:

- material files store texture keys, not backend group ids;
- internal texture entry ids are generated cache/renderer details;
- missing texture keys must produce diagnostics and may use fallback debug state;
- unsupported texture combinations should fail or fallback according to product
  strictness policy.

## Fallback behavior

Fallback behavior keeps content visible and diagnosable.

Fallback may be used when:

- texture key is missing;
- texture file cannot be decoded;
- texture dimensions are invalid;
- texture profile is unsupported by selected backend;
- atlas packing overflows;
- texture array layer limit is exceeded;
- mipmap generation fails;
- renderer capability is missing.

Fallback must preserve:

- material key;
- material field;
- requested texture key;
- fallback debug tint if available;
- diagnostic explaining the failure.

Fallback must not silently replace authored texture keys with renderer-local ids.

## Diagnostics

Validators and DevKit tooling should report at least:

| Diagnostic | Should include |
| --- | --- |
| Missing texture key | material key, field, texture key |
| Invalid texture for backend | texture key, profile, dimensions, reason |
| Incompatible array group | texture key, expected group dimensions/format, actual metadata |
| Atlas overflow | group id, texture key, atlas policy, required/available size |
| Texture array layer limit exceeded | group id, limit, texture count |
| Unsupported strategy | profile, requested strategy, backend capability |
| Unsupported sampling/mip combination | texture key, sampling profile, mip policy |
| Missing mip data | texture key, mip policy |
| Padding/extrusion conflict | texture key, strategy, mip/filter policy |
| Duplicate effective key | key, conflicting package/layer |
| Dedupe notice | keys, shared source hash |
| Stale cache fingerprint | expected fingerprint, actual fingerprint, rebuild hint |
| Renderer-internal id in authored file | file, field, forbidden value |
| Generated cache edited as source | cache path, source category to edit instead |

Diagnostics should name author-facing keys first. Internal ids may appear only as
debug details after the author-facing cause is clear.

## Server/client and compatibility

Some visual assets may be server-required; others may be client-local cosmetic.

Rules:

- server-required texture graph fingerprints should be computed from effective
  authored inputs, not generated atlas coordinates;
- client-local cosmetic texture overrides may change generated backend cache
  without changing authoritative gameplay identity if policy allows;
- generated cache fingerprints are useful for invalidation but are not durable
  compatibility truth by themselves;
- servers should not require clients to report atlas coordinates or array layers;
- clients should validate required texture keys and source/fingerprint policy
  instead of renderer-local layout.

## Relationship to texture authoring

[TEXTURE_AUTHORING_v1.md](TEXTURE_AUTHORING_v1.md) decides whether texture inputs
are valid.

This document decides how valid resolved texture inputs become generated backend
plans.

Example:

~~~text
TEXTURE_AUTHORING_v1:
  example.gems:textures/block/ruby_ore is valid 32x32 voxel_block PNG

TEXTURE_BACKEND_PIPELINE_v1:
  example.gems:textures/block/ruby_ore belongs to voxel_block 32x32 sRGB
  texture_array_2d group and receives an internal layer during generated planning
~~~

The layer is not author-facing.

## Relationship to visual asset model

[VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md) defines textures,
materials, models, effects, and generated atlas/array/load-plan outputs.

This document defines the texture-specific generated backend part of that model.

## Relationship to layered overrides

[LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md) chooses the
effective texture asset before backend planning.

Backend planning sees the final effective texture graph. It does not decide
override policy.

## Relationship to engine renderer work

Engine issue #278 should consume this SDK contract.

This document gives the renderer work:

- stable texture/material key boundaries;
- deterministic plan ordering;
- grouping and strategy vocabulary;
- cache/fingerprint rules;
- fallback/diagnostic expectations;
- explicit prohibition against exposing atlas coordinates or array layers to
  authors.

The engine still owns actual renderer implementation.

## Examples

### Voxel block array group

~~~text
group:
  strategy: texture_array_2d
  profile: voxel_block
  size: 32x32
  color_space: sRGB
  sampling: voxel_nearest
  mip_policy: none

entries:
  freven.vanilla:textures/block/stone
  freven.vanilla:textures/block/dirt
  example.gems:textures/block/ruby_ore
~~~

Generated internal output:

~~~text
array_id: internal.voxel_block.32.srgb.nearest
layers:
  0 -> freven.vanilla:textures/block/stone
  1 -> freven.vanilla:textures/block/dirt
  2 -> example.gems:textures/block/ruby_ore
~~~

The layer numbers are not authored API.

### UI atlas group

~~~text
group:
  strategy: atlas_2d
  profile: ui
  color_space: sRGB
  sampling: ui_linear

entries:
  example.ui:textures/panel/header
  example.ui:textures/icons/warning
~~~

Generated atlas rectangles are cache/debug information only.

### Material binding

~~~text
material:
  example.gems:materials/block/ruby_ore

field:
  base_color_texture

texture key:
  example.gems:textures/block/ruby_ore

generated binding:
  internal texture entry from voxel_block 32x32 texture_array_2d group
~~~

The material remains authored in terms of the texture key.

## Long-term direction

Future versions may add:

- explicit generated-plan file format;
- texture memory budget reports;
- platform-specific backend capability negotiation;
- KTX2/Basis transcode cache policy;
- supplied mip-chain validation;
- atlas padding/extrusion implementation profiles;
- bindless texture backend strategy;
- streaming texture residency policy;
- hot-reload invalidation protocol;
- DevKit visual reports for packing efficiency.

Those extensions should preserve the author-facing key boundary.

## Relationship to shader/effect boundary

Texture backend planning may generate resources consumed by effects, but
shader/effect ownership is defined by
[SHADER_EFFECT_BOUNDARY_v1.md](SHADER_EFFECT_BOUNDARY_v1.md).

Rules:

- effect texture inputs use stable texture keys and declared capabilities;
- backend texture entries, sampler objects, bind groups, and GPU handles remain
  renderer-internal;
- generated shader/cache artifacts are invalid as authored source;
- cache fingerprints may include effect capability inputs, but not generated
  backend ids.
