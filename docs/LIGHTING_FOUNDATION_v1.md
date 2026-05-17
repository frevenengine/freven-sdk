# Lighting Foundation v1

This document defines the Freven rc10 lighting foundation contract for voxel,
block, material, model, and visual authoring.

It builds on:

- [ARCHITECTURE.md](ARCHITECTURE.md): engine / SDK / experience / mod /
  content-pack / standalone-product ownership;
- [PACKAGE_BOUNDARIES.md](PACKAGE_BOUNDARIES.md): manifest, config, content,
  assets, generated cache, and save/world state ownership;
- [VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md): stable visual asset
  identity, dependency graph, validation, and renderer-internal boundaries;
- [MATERIAL_DEFINITIONS_v1.md](MATERIAL_DEFINITIONS_v1.md): material fields such
  as `lighting_model`, emissive maps, emissive factors, and occlusion textures;
- [TEXTURE_AUTHORING_v1.md](TEXTURE_AUTHORING_v1.md): color-space and data-texture
  policy for base color, emissive, and occlusion inputs;
- [TEXTURE_BACKEND_PIPELINE_v1.md](TEXTURE_BACKEND_PIPELINE_v1.md): generated
  backend planning and internal atlas/texture-array boundaries;
- [BLOCK_VISUAL_DEFINITIONS_v1.md](BLOCK_VISUAL_DEFINITIONS_v1.md): block visual
  bindings from gameplay block keys to model/material/tint/render policy;
- [MODEL_ASSET_FORMAT_v1.md](MODEL_ASSET_FORMAT_v1.md): model geometry, normals,
  culling hints, and ambient-occlusion participation;
- [CONTENT_VARIANT_FAMILY_SCHEMA_v1.md](CONTENT_VARIANT_FAMILY_SCHEMA_v1.md):
  generated block/material/model/visual entries and generated light/opacity
  metadata;
- [LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md): deterministic
  visual asset layering and override policy;
- [CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md): semantic add, replace,
  patch, append, disable, compatibility, and diagnostics model;
- [CREATOR_CONTENT_SCHEMA_v1.md](CREATOR_CONTENT_SCHEMA_v1.md): creator-facing
  source schema direction;
- [DATA_DRIVEN_AUTHORING_LAYER_v1.md](DATA_DRIVEN_AUTHORING_LAYER_v1.md):
  practical authoring workflow and shorthand expansion;
- [SHADER_EFFECT_BOUNDARY_v1.md](SHADER_EFFECT_BOUNDARY_v1.md): shader/effect
  ownership boundary for effects that consume lighting capabilities.

The goal is to define a minimal, stable author-facing lighting vocabulary without
exposing renderer slots, shader uniforms, GPU buffers, lightmap coordinates,
runtime block ids, or Vanilla-specific lighting shortcuts.

## Core rule

Lighting declarations are content and scene policy.

A lighting field describes authored lighting intent, visual behavior, or
light-affecting block/material metadata. It is not a renderer light object, not a
shader constant buffer, not a GPU handle, not a generated lightmap coordinate, and
not an implicit gameplay rule.

Author-facing content:

~~~toml
schema = 1
key = "example.glow:materials/block/lamp"

[material]
base_color_texture = "example.glow:textures/block/lamp"
fallback_debug_tint_rgba = "FFD080FF"
lighting_model = "lit"
emissive_rgba = "FFD080FF"
emissive_strength = 1.5

[light]
emits = true
color_rgba = "FFD080FF"
intensity = 12
falloff = "voxel_manhattan"
~~~

Host/backend output:

~~~text
renderer light list
chunk light propagation buffers
baked vertex light values
lightmap/probe cache
shader uniforms
GPU storage buffers
Bevy/wgpu handles
generated cache artifacts
~~~

Only the first model is public SDK vocabulary.

## Goals

- Define minimal lighting vocabulary for rc10 voxel/block scenes.
- Separate visual lighting from authoritative gameplay semantics.
- Support ambient and directional/sun lighting intent.
- Support material lighting models: `lit`, `unlit`, and `pbr_lit`.
- Support emissive material appearance separately from block light emission.
- Support block/voxel light emission metadata without defining final propagation.
- Define light opacity, transmission, and transparent/cutout behavior hooks.
- Define per-face shading and simple voxel ambient-occlusion expectations.
- Keep Vanilla lighting presets in Vanilla or selected standalone content, not in
  engine core.
- Keep renderer-internal light handles, shader bindings, GPU buffers, lightmap
  coordinates, and generated cache out of authored content.
- Give DevKit validation a clear target for diagnostics and explanations.
- Leave enough room for future dynamic lights, probes, lightmaps, shadows, and
  PBR without requiring them in rc10.

## Non-goals

This document does not define:

- final engine lighting implementation;
- final chunk light propagation algorithm;
- final skylight/day-night simulation;
- final shadow maps, GI, probes, or lightmaps;
- final PBR renderer;
- final shader/effect extension ABI;
- final Vanilla sun, torch, glowstone, grass, foliage, water, or glass behavior;
- final entity/item lighting;
- final DevKit CLI commands;
- exact GPU vertex layout, uniform layout, bind groups, or shader modules.

Those remain separate SDK, engine, DevKit, Vanilla, and product issues.

## Terminology

| Term | Meaning |
| --- | --- |
| Scene ambient light | Coarse base illumination for a scene or view. |
| Directional light | Infinite light with a direction, typically sun/moon style. |
| Sky light | World/voxel light influenced by open sky or environment policy. |
| Block light | Voxel-local light emitted by content such as lamps or fire. |
| Emissive material | Material that appears self-lit. It may or may not emit block light. |
| Light opacity | How much a block/material prevents light transmission. |
| Light transmission | How much light passes through a block/material. |
| Per-face shading | Shading based on face normal or model normal. |
| Ambient occlusion | Local darkening from nearby geometry/voxels. |
| Lightmap/probe | Generated renderer data. Not authored content identity. |
| Lighting model | Material shading intent such as `lit`, `unlit`, or `pbr_lit`. |

## Ownership

Lighting crosses several systems, so ownership must stay explicit.

| Owner | Owns |
| --- | --- |
| SDK docs | Author-facing vocabulary, content fields, validation expectations, compatibility classes. |
| Engine/runtime | Actual light propagation, renderer implementation, shader inputs, generated caches. |
| Vanilla / standalone game | Chosen sun/sky presets, torch strengths, block light library, art direction. |
| Mods/content packs | Authored material/light metadata and allowed overrides. |
| Server/selected experience | Which light-affecting metadata is authoritative or required. |
| Generated cache | Lightmaps, probes, baked vertex light, chunk light buffers, renderer-specific artifacts. |

Rules:

- SDK defines the vocabulary, not the runtime algorithm.
- Engine may implement a subset as long as unsupported fields produce diagnostics
  or deterministic fallback.
- Vanilla lighting values are content/library choices, not engine defaults.
- Cosmetic client-local lighting changes must not alter authoritative gameplay
  unless selected policy explicitly permits it.
- Generated lighting artifacts are rebuildable output and must not be authored as
  stable content identity.

## Scene lighting

A selected experience may define scene-level lighting policy.

Conceptual scene lighting content:

~~~toml
schema = 1
key = "example.world:lighting/overworld"

[scene_lighting]
ambient_rgba = "FFFFFFFF"
ambient_intensity = 0.25

[scene_lighting.directional.sun]
direction = [-0.35, -0.8, -0.25]
color_rgba = "FFF3D0FF"
intensity = 1.0
shadow_policy = "none"
~~~

v1 canonical fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `ambient_rgba` | `RRGGBBAA` | Ambient light color. |
| `ambient_intensity` | number `>= 0.0` | Ambient multiplier. |
| `directional.*.direction` | vec3 | Author-facing world/scene direction. |
| `directional.*.color_rgba` | `RRGGBBAA` | Directional light color. |
| `directional.*.intensity` | number `>= 0.0` | Directional light multiplier. |
| `directional.*.shadow_policy` | enum | Future-compatible shadow hint. |

Accepted v1 `shadow_policy` values:

| Value | Meaning |
| --- | --- |
| `none` | No authored shadow requirement. |
| `simple` | Simple renderer-supported shadows if available. |
| `required` | Selected stack requires an implementation or diagnostic. |

Rules:

- scene lighting keys are content keys;
- scene lighting is selected by experience/product/world policy;
- direction vectors are author-facing semantic values, not shader constants;
- unsupported directional or shadow fields must produce diagnostics or fallback;
- renderer handles, cascades, shadow-map sizes, and uniform-buffer bindings are
  engine/backend details.

## Material lighting model

Materials already declare `lighting_model`.

Accepted v1 values remain:

| Value | Meaning |
| --- | --- |
| `lit` | Default engine-lit material. |
| `unlit` | Ignores scene lighting; useful for UI, debug, decals, and emissive-only visuals. |
| `pbr_lit` | PBR-ready lit material using metallic/roughness-style fields where supported. |

Rules:

- `lit` participates in scene, block, and per-face lighting where implemented;
- `unlit` should still respect alpha/render-layer behavior but not be darkened by
  scene lighting;
- `pbr_lit` records author intent even if the current renderer falls back to
  `lit`;
- unsupported lighting models are diagnostics;
- lighting model is material content, not renderer pipeline id;
- if a material is server-required or authoritative, `lighting_model` may
  participate in compatibility fingerprints.

## Emissive materials and block light emission

Emissive appearance and light emission are separate.

Material emissive fields describe how the surface looks:

~~~toml
[material]
emissive_texture = "example.glow:textures/block/lamp_emissive"
emissive_rgba = "FFD080FF"
emissive_strength = 2.0
~~~

Block or visual light fields describe whether the content emits light into the
world/voxel lighting system:

~~~toml
[light]
emits = true
color_rgba = "FFD080FF"
intensity = 12
falloff = "voxel_manhattan"
~~~

Rules:

- an emissive material may glow visually without emitting block light;
- a block may emit light without a separate emissive texture;
- light emission affects selected-stack/world identity when the experience treats
  lighting as gameplay-visible or server-required;
- `intensity` is an author-facing scalar or discrete level, not a renderer lumen
  guarantee;
- actual propagation, falloff, range clamp, chunk updates, and networking belong
  to engine/runtime implementation;
- unsupported emission fields must produce diagnostics or deterministic fallback.

Suggested v1 light emission fields:

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `light.emits` | bool | `false` | Whether this content emits block/voxel light. |
| `light.color_rgba` | `RRGGBBAA` | `FFFFFFFF` | Emitted light color. |
| `light.intensity` | integer/number | `0` | Author-facing light strength. |
| `light.falloff` | enum | selected policy | Propagation/falloff policy key. |
| `light.source_shape` | enum | `block` | Future hook for point/area/model-local emission. |

Suggested v1 `falloff` values:

| Value | Meaning |
| --- | --- |
| `none` | No propagated light. |
| `voxel_manhattan` | Simple block-grid falloff suitable for Minecraft-like voxel light. |
| `renderer_only` | Visual-only light hint, not authoritative voxel light. |
| `custom:<namespace:path>` | Selected-stack custom policy key, if allowed. |

## Light opacity and transmission

Blocks, materials, or visuals may declare light transmission intent.

Conceptual solid metadata:

~~~toml
[light]
opacity = 1.0
transmission = 0.0
~~~

Conceptual glass metadata:

~~~toml
[light]
opacity = 0.0
transmission = 0.85
tint_transmission = true
~~~

Suggested fields:

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `light.opacity` | number `0.0..1.0` | derived from block/material policy | How strongly this content blocks light. |
| `light.transmission` | number `0.0..1.0` | derived from opacity | How strongly light passes through. |
| `light.tint_transmission` | bool | `false` | Whether transmitted light may be tinted by material/color. |
| `light.skylight_policy` | enum | selected policy | How this content interacts with sky light. |

Accepted v1 `skylight_policy` values:

| Value | Meaning |
| --- | --- |
| `default` | Selected stack derives behavior from opacity/render layer. |
| `blocks` | Blocks sky light. |
| `passes` | Allows sky light to pass. |
| `attenuates` | Reduces sky light by declared opacity/transmission. |

Rules:

- opaque full blocks normally block light;
- cutout materials may be treated as mostly opaque or partially transmitting by
  selected policy;
- transparent materials may pass or attenuate light;
- glass tinting transmitted light is a content/art-direction feature, not an
  engine assumption;
- gameplay opacity and visual transparency are related but not identical;
- final propagation semantics belong to engine/runtime and selected experience
  policy.

## Transparent and cutout lighting behavior

Transparent and cutout render layers must not silently define light behavior.

Rules:

- `render_layer = "transparent"` means blended rendering, not automatic light
  transmission;
- `render_layer = "cutout"` means alpha test rendering, not automatic leaf-like
  skylight behavior;
- light transmission should be declared through explicit light/opacity metadata
  or selected policy;
- diagnostics should warn when a visual/material combines surprising values, such
  as transparent render layer with full light opacity, or opaque render layer with
  high transmission;
- transparent sorting, refraction, colored glass, volumetric fog, and caustics are
  future renderer features, not v1 requirements.

## Per-face shading

Voxel/block renderers often use simple face-direction shading.

Suggested v1 face shading policy:

| Value | Meaning |
| --- | --- |
| `none` | Do not apply face-normal darkening. |
| `voxel_cardinal` | Apply simple cardinal-direction face shading. |
| `model_normals` | Use model/mesh normals where supported. |
| `custom:<namespace:path>` | Selected-stack custom policy key, if allowed. |

Conceptual field:

~~~toml
[visual.shading]
face_shading = "voxel_cardinal"
~~~

Rules:

- face shading is visual policy;
- the engine may bake it into vertex colors, use normals, or apply shader logic;
- authored data must not name shader locations, vertex attributes, or GPU buffers;
- model normals and generated mesh normals are renderer output, not authored
  renderer state;
- selected product/experience policy may choose the default.

## Ambient occlusion

Model v1 already reserves `model.ambient_occlusion`.

Lighting foundation v1 gives that field a shared meaning:

~~~toml
[model]
ambient_occlusion = true
~~~

Suggested AO policies:

| Value | Meaning |
| --- | --- |
| `none` | No AO. |
| `voxel_simple` | Simple local voxel AO from neighbor occupancy. |
| `model_simple` | Simple model/part-local AO where supported. |
| `baked` | Imported/baked AO data where supported. |
| `custom:<namespace:path>` | Selected-stack custom policy key, if allowed. |

Rules:

- AO is visual shading, not gameplay collision;
- AO participation may be declared by model/material/visual policy;
- generated AO vertex values, AO textures, and lightmap channels are generated
  cache/backend details;
- transparent/cutout AO behavior should be explicit or selected-policy-derived;
- unsupported AO policy must produce diagnostics or deterministic fallback.

## Sky light, block light, and future dynamic lights

Lighting v1 reserves a small vocabulary without requiring full implementation.

| Light family | v1 status |
| --- | --- |
| Ambient scene light | Contract defined. Implementation may be simple uniform/fallback. |
| Directional/sun light | Contract defined. Shadows optional/future. |
| Sky light | Hook defined. Final propagation/day-night behavior is future work. |
| Block light | Emission/opacity hooks defined. Final propagation is future work. |
| Emissive surfaces | Material appearance contract defined. |
| Dynamic entity lights | Future-compatible hook only. |
| Lightmaps/probes/GI | Future generated-cache/backend work. |

Future dynamic light declarations should use stable content keys or selected-stack
policy keys. They must not expose renderer entity ids, GPU handles, bind groups,
or transient runtime ids as authored content.

## Compatibility and fingerprints

Lighting fields may be cosmetic, server-required, or authoritative depending on
selected product/experience/server policy.

For server-required or authoritative lighting content, fingerprints should include
the authored semantic fields that affect visible or gameplay-relevant lighting,
such as:

- scene lighting key and schema version;
- ambient/directional color and intensity;
- material `lighting_model`;
- emissive fields;
- block light emission fields;
- light opacity/transmission fields;
- face shading and AO policy keys;
- declared compatibility/authority class.

Fingerprints should not include:

- renderer light handles;
- shader pipeline ids;
- uniform-buffer layout;
- GPU handles;
- generated lightmap coordinates;
- generated cache paths;
- filesystem traversal order;
- backend-specific precision or packing.

Rules:

- client-local cosmetic lighting packs may be allowed only by policy;
- server-required lighting metadata should be checked before joining/running;
- authoritative light-affecting gameplay must be represented as explicit
  gameplay/world/content state, not hidden visual override;
- diagnostics must explain which fields are cosmetic, server-required, or
  authoritative.

## Patch and override behavior

Lighting fields are semantic content, not raw renderer state.

Examples:

~~~toml
[[content_patches]]
op = "patch"
kind = "material"
target = "freven.vanilla:materials/block/torch"
path = "material.emissive_strength"
value = 1.25
authority = "selected_stack"
reason = "selected visual balance"
~~~

~~~toml
[[content_patches]]
op = "patch"
kind = "block"
target = "freven.vanilla:blocks/glass"
path = "light.transmission"
value = 0.9
authority = "selected_stack"
reason = "clear glass pack"
~~~

Rules:

- patching emissive strength is a material/content patch;
- patching light emission, opacity, or transmission is content patching;
- swapping emissive texture bytes is an asset override;
- rebuilding lightmaps/probes/chunk light buffers is generated cache behavior;
- no patch may write renderer light handles, shader constants, GPU buffers,
  lightmap coordinates, or generated cache paths into content.

## Validation diagnostics

Validators should report at least:

| Diagnostic | Should include |
| --- | --- |
| Invalid lighting key | file path, key, expected `namespace:path` |
| Invalid color | key, field, expected `RRGGBBAA` |
| Invalid intensity/range | key, field, accepted range |
| Invalid lighting model | material key, field, accepted values |
| Unsupported lighting model | material key, runtime capability, fallback |
| Emissive field unsupported | material key, field, runtime capability |
| Emissive material without emission | material key, explanation if likely confusing |
| Light emission without visual emissive hint | block/material key, suggested fix |
| Invalid light opacity/transmission | content key, field, accepted range |
| Transparent light mismatch | material/visual key, render layer, opacity/transmission |
| Cutout light policy missing | material/visual key, selected policy |
| Unsupported shadow policy | lighting key, field, runtime capability |
| Unsupported AO policy | model/visual key, field, fallback |
| Renderer-internal id used | key, field, forbidden value |
| Generated cache referenced | key, field, forbidden path/value |

Diagnostics should be actionable for DevKit users and should not require knowing
engine internals.

## Current engine bridge

The current engine may implement only a subset of this contract.

A minimal rc10 bridge may support:

| Lighting Foundation v1 | Minimal engine bridge |
| --- | --- |
| `lighting_model = "lit"` | Apply simple directional/ambient or existing debug directional lighting. |
| `lighting_model = "unlit"` | Skip scene lighting where supported, otherwise warn/fallback. |
| `lighting_model = "pbr_lit"` | Treat as `lit` with diagnostic or capability report. |
| `emissive_rgba` / `emissive_strength` | Preserve metadata; render fallback may ignore or approximate. |
| `light.emits` | Preserve metadata; propagation may be future work. |
| `light.opacity` / `transmission` | Preserve metadata for future propagation and compatibility. |
| `model.ambient_occlusion` | Generate simple voxel AO where supported or fallback. |

Rules:

- unsupported fields must not be silently interpreted as renderer ids;
- fallback behavior should be deterministic;
- DevKit should be able to explain what the current runtime supports;
- engine implementation details remain outside SDK docs.

## Relationship to existing visual docs

- Material definitions own material-level fields such as `lighting_model`,
  emissive textures, emissive colors, and occlusion textures.
- Block visual definitions own block-to-model/material binding and may carry
  visual/light policy hooks.
- Model asset format owns geometry, normals, culling hints, and AO participation.
- Texture authoring owns color-space and channel policy for emissive/occlusion
  textures.
- Texture backend pipeline owns generated texture-array/atlas/lightmap/backend
  planning.
- Content variant families may generate light/opacity metadata like any other
  semantic content.
- Shader/effect extension policy is a separate follow-up boundary.

If documents disagree, the more specific owner wins for that field, while this
document owns the shared lighting vocabulary and cross-system semantics.

## Long-term direction

Lighting v1 is deliberately small.

Future versions may add:

- day/night cycles and sky models;
- colored skylight and weather;
- real block light propagation contract;
- dynamic entity and item lights;
- shadows and cascades;
- baked lightmaps;
- reflection probes;
- irradiance probes;
- volumetric fog;
- physically based light units;
- shader/effect extension hooks;
- marketplace/client-local lighting policy.

Those additions should extend the author-facing schema without exposing renderer
internals.

## Relationship to shader/effect boundary

Lighting vocabulary is consumed by renderer effects, but shader/effect ownership
is defined by [SHADER_EFFECT_BOUNDARY_v1.md](SHADER_EFFECT_BOUNDARY_v1.md).

Rules:

- lighting fields are semantic content/scene policy;
- effects may declare lighting-related capabilities such as `surface_lighting`,
  `emissive`, or `occlusion`;
- shader uniforms, bind groups, generated lightmaps, and renderer light handles
  are backend/generated output;
- raw shader effects must not become hidden gameplay light authority.

## Conformance fixtures

Lighting-related conformance examples live under
`fixtures/visual_content_schema_v1/valid/`.

The fixture set intentionally separates emissive material appearance from actual
light emission metadata. Consumers should use this split when validating authored
content and when building runtime/load-plan representations.
