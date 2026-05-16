# Shader / Effect Extension Boundary v1

This document defines the Freven rc10 boundary for shader and visual-effect
extension ownership.

It builds on:

- [ARCHITECTURE.md](ARCHITECTURE.md): engine / SDK / experience / mod /
  content-pack / standalone-product ownership;
- [PACKAGE_BOUNDARIES.md](PACKAGE_BOUNDARIES.md): manifest, config, content,
  assets, generated cache, and save/world state ownership;
- [VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md): stable visual asset
  identity, shader/effect asset category, dependency graph, validation, and
  renderer-internal boundaries;
- [MATERIAL_DEFINITIONS_v1.md](MATERIAL_DEFINITIONS_v1.md): material fields,
  PBR-ready vocabulary, lighting model, and renderer-internal boundaries;
- [TEXTURE_AUTHORING_v1.md](TEXTURE_AUTHORING_v1.md): texture profile and
  effect-sprite texture policy;
- [TEXTURE_BACKEND_PIPELINE_v1.md](TEXTURE_BACKEND_PIPELINE_v1.md): generated
  backend planning and internal texture/material binding boundaries;
- [BLOCK_VISUAL_DEFINITIONS_v1.md](BLOCK_VISUAL_DEFINITIONS_v1.md): block visual
  bindings from gameplay block keys to model/material/tint/render policy;
- [MODEL_ASSET_FORMAT_v1.md](MODEL_ASSET_FORMAT_v1.md): model hooks, future
  animation/effect hooks, and renderer-internal mesh boundaries;
- [LIGHTING_FOUNDATION_v1.md](LIGHTING_FOUNDATION_v1.md): lighting vocabulary,
  shader/effect follow-up boundary, and generated lighting/backend boundaries;
- [LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md): deterministic
  visual asset layering and server/client cosmetic rules;
- [CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md): semantic add, replace,
  patch, append, disable, compatibility, and diagnostics model;
- [CREATOR_CONTENT_SCHEMA_v1.md](CREATOR_CONTENT_SCHEMA_v1.md): creator-facing
  source schema direction;
- [DATA_DRIVEN_AUTHORING_LAYER_v1.md](DATA_DRIVEN_AUTHORING_LAYER_v1.md):
  practical authoring workflow and shorthand expansion.

The goal is to make shader ownership explicit without exposing a broad renderer
plugin ABI in rc10.

## Core rule

Shader and effect declarations are author-facing visual intent.

They are not renderer pipelines, not bind groups, not shader module handles, not
GPU buffers, not generated cache, not a way to mutate gameplay truth, and not a
free-form backend plugin API.

Author-facing content:

~~~toml
schema = 1
key = "example.weather:effects/leaves_wind"

[effect]
kind = "surface"
entry = "freven.builtin:effects/voxel_surface"
fallback = "freven.builtin:effects/default_lit"
capabilities = ["surface_color", "vertex_wind"]

[effect.inputs]
wind_strength = { type = "float", default = 0.25, min = 0.0, max = 2.0 }
tint_mix = { type = "float", default = 1.0, min = 0.0, max = 1.0 }
~~~

Host/backend output:

~~~text
shader module
pipeline layout
bind group layout
GPU bind group
uniform/storage buffer
material pipeline id
specialization constant
generated shader cache
backend-specific shader translation
~~~

Only the first model is stable public SDK vocabulary.

## Goals

- Define who owns shader/effect decisions.
- Keep renderer resource layout private to engine/runtime.
- Let experiences and standalone games choose visual style through named
  effects, material choices, post-processing policy, and selected-stack config.
- Let content packs and mods request declared visual effects without gaining
  arbitrary renderer access.
- Define safe capability/fallback/diagnostic vocabulary.
- Keep Vanilla visual style out of engine core.
- Preserve future room for trusted renderer plugins without making raw shader
  ABI part of rc10.
- Keep shader/effect choices compatible with layered asset overrides and
  server/client cosmetic policy.
- Make DevKit diagnostics actionable.

## Non-goals

This document does not define:

- final WGSL/HLSL/GLSL source ABI;
- final renderer plugin ABI;
- final material graph or node graph schema;
- final post-processing graph implementation;
- final particle/effect system;
- final animation/effect runtime;
- final hot-reload implementation;
- final shader compiler implementation;
- exact bind group layouts;
- exact GPU vertex layouts;
- exact shader module paths or generated cache paths;
- marketplace trust UI.

Those remain separate SDK, engine, DevKit, product, and trust-policy issues.

## Ownership

| Owner | Owns |
| --- | --- |
| SDK docs | Public vocabulary: effect keys, effect kinds, declared inputs, capability names, fallback requirements, compatibility rules, diagnostics. |
| Engine/runtime | Shader loading, compilation, validation, resource layout, bind groups, pipeline creation, fallback shader, generated caches, backend portability. |
| Experience / standalone game | Selected visual style, allowed effect set, material/effect choices, post-processing choices, style presets. |
| Vanilla | Vanilla-specific effect library and style choices. |
| Mods/content packs | Semantic effect declarations and safe requests inside allowed capabilities. |
| Server/selected stack | Which shader/effect choices are cosmetic, server-required, denied, or trusted. |
| Generated cache | Compiled shaders, backend translations, pipeline fingerprints, reflection data, derived artifacts. |

Rules:

- engine owns safety and renderer contracts;
- experiences own style and effect selection;
- SDK defines vocabulary and validation expectations;
- Vanilla effects are content/product assets, not engine defaults;
- raw shader source execution is trusted/advanced policy, not normal content;
- renderer handles and generated artifacts must not appear in authored content.

## Effect identity

A shader/effect has a stable visual asset key:

~~~text
namespace:effects/<domain>/<name>
~~~

Examples:

~~~text
freven.builtin:effects/default_lit
freven.builtin:effects/default_unlit
freven.vanilla:effects/leaves_wind
freven.vanilla:effects/water_surface
example.weather:effects/fog_overlay
example.magic:effects/glow_pulse
example.ui:effects/panel_blur
~~~

Rules:

- the namespace owns the effect identity;
- the path identifies semantic visual behavior, not backend pipeline ids;
- keys must not encode shader stage, bind group number, GPU handle, pipeline id,
  cache path, or renderer object id;
- selected stack resolution must produce one effective declaration per key;
- layered overrides and content patches apply to effect declarations as semantic
  content, not raw generated shader cache.

## Effect kinds

rc10 recognizes a conservative vocabulary.

| Kind | Meaning |
| --- | --- |
| `surface` | Material/surface effect for blocks, models, items, entities, or UI surfaces. |
| `post_process` | Fullscreen or view-level effect chosen by product/experience policy. |
| `particle` | Future particle/sprite/billboard effect. |
| `decal` | Future projected/surface decal effect. |
| `ui` | Future UI-specific effect. |
| `debug` | Debug/diagnostic visualization effect. |
| `builtin_alias` | Alias to a built-in engine-supported effect hook. |
| `custom_trusted` | Future trusted renderer extension, denied by default in normal content. |

Rules:

- kind names are semantic SDK vocabulary, not renderer pipeline enums;
- unsupported kinds must produce diagnostics or deterministic fallback;
- `custom_trusted` must require explicit trust/capability policy;
- normal mods should target `surface`, `post_process`, `particle`, `decal`,
  `ui`, `debug`, or `builtin_alias`, not arbitrary renderer plugins.

## Built-in hooks

The engine may expose built-in hooks as stable effect keys.

Suggested rc10 built-ins:

| Key | Purpose |
| --- | --- |
| `freven.builtin:effects/default_lit` | Default lit surface fallback. |
| `freven.builtin:effects/default_unlit` | Default unlit surface fallback. |
| `freven.builtin:effects/default_cutout` | Default alpha-tested surface fallback. |
| `freven.builtin:effects/default_transparent` | Default blended surface fallback. |
| `freven.builtin:effects/debug_missing` | Visible missing/unsupported effect diagnostic fallback. |

Rules:

- built-in effect keys are stable public names;
- their renderer implementation is private;
- shader source, bind groups, vertex layout, and pipeline layout remain private;
- products may replace style by selecting different effect keys where policy
  allows it;
- engine must provide deterministic fallback for missing or unsupported effects.

## Effect declarations

A v1 effect declaration describes requirements and safe inputs.

Conceptual declaration:

~~~toml
schema = 1
key = "example.magic:effects/glow_pulse"

[effect]
kind = "surface"
fallback = "freven.builtin:effects/default_lit"
capabilities = ["surface_color", "emissive", "time_phase"]

[effect.inputs]
pulse_speed = { type = "float", default = 1.0, min = 0.0, max = 16.0 }
pulse_color = { type = "rgba", default = "80BFFFFF" }

[effect.source]
language = "wgsl"
path = "assets/shaders/glow_pulse.wgsl"
trust = "trusted_package_only"
~~~

Important: `[effect.source]` is future-compatible. rc10 may validate and preserve
it without executing arbitrary source.

Canonical fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `schema` | integer | Effect declaration schema version. |
| `key` | `namespace:path` | Stable effect identity. |
| `effect.kind` | enum | Semantic effect kind. |
| `effect.fallback` | effect key | Required fallback when unsupported. |
| `effect.capabilities` | list | Declared safe capability requirements. |
| `effect.inputs` | map | Typed author-facing inputs. |
| `effect.source.language` | enum/string | Future trusted source language. |
| `effect.source.path` | package-local asset path | Future trusted shader source path. |
| `effect.source.trust` | enum | Trust/capability requirement for raw source. |

Rules:

- `fallback` should be required for non-built-in effects;
- inputs are semantic values, not uniform-buffer offsets;
- source paths are package-local assets, not global filesystem paths;
- raw shader source is not normal authoring unless trust policy allows it;
- unknown fields are diagnostics or preserved only under explicit extension policy.

## Capabilities

Capabilities describe what an effect needs from the renderer/content pipeline.

Suggested capability names:

| Capability | Meaning |
| --- | --- |
| `surface_color` | Needs base color / albedo input. |
| `surface_normal` | Needs normals or normal map support. |
| `surface_pbr` | Needs PBR-ready material inputs. |
| `surface_alpha` | Needs alpha/cutout/blend behavior. |
| `surface_tint` | Needs visual tint input. |
| `surface_lighting` | Needs lighting inputs from lighting foundation. |
| `emissive` | Needs emissive color/texture support. |
| `occlusion` | Needs AO/occlusion input. |
| `vertex_wind` | Needs model/vertex animation hook. |
| `time_phase` | Needs deterministic visual time input. |
| `camera_depth` | Needs depth texture access. |
| `scene_color` | Needs color buffer access for post-processing. |
| `debug_overlay` | Needs debug visualization channel. |

Rules:

- capabilities are author-facing requirements, not bind group names;
- engine maps capabilities to backend resources privately;
- unsupported capabilities must produce diagnostics or fallback;
- server-required capability fingerprints use capability names, not pipeline ids;
- capabilities must not grant gameplay authority by themselves.

## Effect inputs

Effect inputs are typed semantic parameters.

Accepted v1 input types:

| Type | Meaning |
| --- | --- |
| `bool` | Boolean toggle. |
| `int` | Integer scalar with optional min/max. |
| `float` | Floating-point scalar with optional min/max. |
| `vec2` / `vec3` / `vec4` | Numeric vectors. |
| `rgba` | Color as `RRGGBBAA`. |
| `texture_key` | Stable texture asset key. |
| `material_key` | Stable material key. |
| `effect_key` | Stable effect key. |
| `enum` | Closed set of string choices. |

Rules:

- effect inputs are semantic data;
- inputs must have defaults or be explicitly required;
- ranges and enum values should be validated by tooling;
- input names must not be shader uniform names unless separately declared as a
  future trusted mapping;
- uniform offsets, binding numbers, specialization constants, and GPU buffer
  layouts are engine/backend details.

## Material and block visual references

Materials and visuals may reference an effect key.

Conceptual material:

~~~toml
schema = 1
key = "example.magic:materials/block/glowing_stone"

[material]
base_color_texture = "example.magic:textures/block/glowing_stone"
lighting_model = "lit"
effect = "example.magic:effects/glow_pulse"
fallback_debug_tint_rgba = "80BFFFFF"
~~~

Conceptual block visual:

~~~toml
schema = 1
key = "example.weather:visuals/block/windy_leaves"

[visual]
target = "example.weather:blocks/windy_leaves"
model = "example.weather:models/block/leaves"
effect = "example.weather:effects/leaves_wind"
~~~

Rules:

- material/visual effect references are stable keys;
- missing effect keys are diagnostics;
- effects must declare required capabilities;
- selected product/server policy decides whether the effect is cosmetic,
  server-required, denied, or trusted;
- renderer pipeline ids must never be authored.

## Post-processing

Post-processing is selected by product/experience policy, not by arbitrary block
or item content.

Conceptual experience policy:

~~~toml
[visual.post_process]
chain = [
  "freven.vanilla:effects/tonemap_default",
  "example.weather:effects/fog_soft",
]
fallback = "freven.builtin:effects/default_unlit"
~~~

Rules:

- post-process effects are view/product policy, not gameplay block metadata;
- client-local post-processing packs may be cosmetic only if allowed by policy;
- server-required visual experiences may restrict the post-process chain;
- scene color, depth, motion vectors, and camera buffers are capabilities, not
  bind groups;
- output formats, render graph nodes, and intermediate textures are backend
  details.

## Trust and safety

Shader/effect extension is a trust boundary.

Risky features:

- arbitrary shader source execution;
- custom renderer plugins;
- custom bind group layouts;
- arbitrary storage buffer access;
- compute shader dispatch;
- screen/depth buffer access;
- hidden gameplay-relevant visualization;
- bypassing server-required visual restrictions;
- loading generated cache as authored source.

Rules:

- normal content may request named effects and declared capabilities;
- raw shader source should require explicit trusted package/product policy;
- custom renderer plugins should require a separate native/unsafe trust model;
- server-required experiences may deny client-local shader/effect overrides;
- diagnostics must explain whether denial is because of missing capability,
  unsupported backend, untrusted source, or selected-stack policy.

## Compatibility and fingerprints

Shader/effect declarations may be cosmetic, server-required, or trusted depending
on selected policy.

Server-required fingerprints should include:

- effective effect key;
- declaration schema version;
- effect kind;
- declared capabilities;
- declared input names, types, defaults, and selected values;
- fallback effect key;
- referenced texture/material/effect keys;
- source asset hash only when raw source is trusted and part of policy;
- trust class and selected-stack authority class.

Fingerprints should not include:

- compiled shader module id;
- pipeline id;
- bind group layout id;
- uniform offset;
- GPU handle;
- generated cache path;
- backend translation artifact path;
- filesystem traversal order;
- hot-reload timestamp.

Rules:

- cosmetic shader/effect changes may be client-local only when policy allows;
- server-required effect graphs should be checked before join/run;
- generated shader cache may be fingerprinted for invalidation but is not
  authored content or durable save/world state.

## Patch and override behavior

Shader/effect declarations are semantic content.

Examples:

~~~toml
[[content_patches]]
op = "replace"
kind = "effect"
target = "freven.vanilla:effects/leaves_wind"
value = "example.weather:effects/leaves_wind_soft"
authority = "selected_stack"
reason = "visual style pack"
~~~

~~~toml
[[content_patches]]
op = "patch"
kind = "effect"
target = "example.magic:effects/glow_pulse"
path = "effect.inputs.pulse_speed.default"
value = 0.75
authority = "selected_stack"
reason = "slower pulse"
~~~

Rules:

- replacing an effect key is content patching;
- replacing shader source bytes is an asset override;
- rebuilding compiled shader modules is generated cache behavior;
- no patch may write bind group numbers, pipeline ids, GPU handles, generated
  cache paths, or shader module handles into content.

## Diagnostics

Validators should report at least:

| Diagnostic | Should include |
| --- | --- |
| Invalid effect key | file path, key, expected `namespace:path` |
| Missing effect dependency | referencing material/visual/effect key, missing key |
| Unsupported effect kind | effect key, kind, supported kinds |
| Missing fallback | effect key, required fallback field |
| Unsupported capability | effect key, capability, runtime/product support |
| Invalid input type | effect key, input name, type, accepted types |
| Invalid input range/default | effect key, input name, range/default |
| Raw source denied | effect key, source path, required trust policy |
| Unsupported shader language | effect key, language, supported trusted languages |
| Renderer-internal field used | key, field, forbidden value |
| Generated cache referenced | key, field, forbidden path/value |
| Server-required effect mismatch | expected key/hash, actual key/hash, policy |
| Client-local effect denied | effect key, policy reason |
| Fallback used | effect key, fallback key, reason |

Diagnostics should point to authored files and stable keys, not generated cache or
renderer internals.

## Current rc10 bridge

The current rc10 bridge may support only a subset.

| Boundary field | Minimal behavior |
| --- | --- |
| Built-in effect keys | Preserve/diagnose and map to engine defaults where available. |
| Material/visual effect key | Preserve as semantic metadata or diagnostic until engine support lands. |
| Declared capabilities | Validate vocabulary and report unsupported capabilities. |
| Raw source fields | Deny or preserve as trusted future metadata; do not execute by default. |
| Post-process chain | Preserve as selected product/experience policy or diagnostic. |
| Fallback key | Required for unsupported non-built-in effects. |

Rules:

- fallback behavior must be deterministic;
- unsupported effects must not silently become renderer ids;
- engine implementation can evolve without changing author-facing keys;
- DevKit should explain effective effect graph and fallback decisions.

## Relationship to existing visual docs

- Visual asset model owns stable asset identity and dependency graph.
- Material definitions may reference effect keys but do not define shader ABI.
- Texture docs own source texture profiles and effect-sprite texture policy.
- Texture backend pipeline owns generated backend/cache planning.
- Block visual definitions may reference effect keys but not renderer internals.
- Model format may define animation/effect hooks but not shader resource layout.
- Lighting foundation owns lighting vocabulary; shader/effect boundary owns how
  effects request lighting-related capabilities.
- Layered overrides and content patch/merge define composition semantics.
- Creator/data-driven docs may expose friendly shorthand that compiles to effect
  declarations and references.

If documents disagree, the more specific owner wins for that field, while this
document owns shader/effect extension boundary and trust semantics.

## Long-term direction

Future versions may add:

- trusted WGSL source ABI;
- material graph or node graph schema;
- post-processing graph schema;
- particle/effect schema;
- effect hot reload;
- shader reflection/cache metadata;
- renderer plugin trust model;
- compute effect model;
- marketplace trust UI;
- per-product effect capability profiles.

Those additions should extend the author-facing schema without exposing renderer
internals as stable content values.
