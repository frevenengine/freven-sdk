# Data-Driven Content Authoring Layer v1

This document defines the practical Freven data-driven content authoring layer
for simple blocks, items, recipes, entities, visuals, materials, and behavior
bindings.

It builds on:

- [ARCHITECTURE.md](ARCHITECTURE.md): engine / SDK / experience / mod /
  content-pack / standalone-product ownership;
- [PACKAGE_BOUNDARIES.md](PACKAGE_BOUNDARIES.md): manifest, config, content,
  assets, generated cache, and save/world state ownership;
- [VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md): visual asset identity,
  categories, dependency graph, validation, and renderer-backend boundaries;
- [MATERIAL_DEFINITIONS_v1.md](MATERIAL_DEFINITIONS_v1.md): canonical
  data-driven material definition schema;
- [TEXTURE_AUTHORING_v1.md](TEXTURE_AUTHORING_v1.md): texture size, sampling,
  mipmap, alpha, color-space, and validation policy;
- [TEXTURE_BACKEND_PIPELINE_v1.md](TEXTURE_BACKEND_PIPELINE_v1.md):
  generated texture backend planning and internal atlas/array boundaries;
- [BLOCK_VISUAL_DEFINITIONS_v1.md](BLOCK_VISUAL_DEFINITIONS_v1.md): block visual
  bindings from gameplay block keys to model/material/tint/render policy;
- [MODEL_ASSET_FORMAT_v1.md](MODEL_ASSET_FORMAT_v1.md): model asset
  declarations for cube, cuboid, item, entity/static, material-slot, UV, and
  transform authoring;
- [CONTENT_VARIANT_FAMILY_SCHEMA_v1.md](CONTENT_VARIANT_FAMILY_SCHEMA_v1.md):
  deterministic content family expansion for variants, generated keys,
  allow/skip lists, and per-variant overrides;
- [LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md): visual asset
  layer ordering and override policy;
- [CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md): semantic add, replace,
  patch, append, disable, compatibility, and diagnostics model;
- [CREATOR_CONTENT_SCHEMA_v1.md](CREATOR_CONTENT_SCHEMA_v1.md): creator-friendly
  schema direction for common content files;
- [SHADER_EFFECT_BOUNDARY_v1.md](SHADER_EFFECT_BOUNDARY_v1.md): shader/effect
  ownership boundary for friendly effect shorthand, named effects, capability
  checks, fallbacks, trust, and diagnostics.

The goal is to describe the first practical authoring workflow that new modders,
content packs, Vanilla-like mods, and simple standalone games should be able to
use without writing Rust for every piece of content.

## Core rule

The authoring layer is a friendly source layer.

It does not create a second content model. Authoring files compile into the same
semantic content entries and patch/merge operations defined by
[CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md).

A beginner-friendly file may be concise, but the resolved result must still be
explicit, namespaced, validated, deterministic, and diagnosable.

## Goals

- Let simple content be authored without Rust.
- Make the common block/item/recipe/entity path short and readable.
- Keep all identities stable and namespaced after shorthand expansion.
- Keep visuals and assets separate from runtime renderer internals.
- Allow data-only content to reference advanced Wasm/script behavior providers.
- Make DevKit validation point to the exact friendly source file and field.
- Support Vanilla mods, content packs, total conversions, and zero-Vanilla
  standalone games.
- Keep generated cache, save/world state, and runtime mutation out of source
  authoring files.

## Non-goals

This document does not define:

- final engine/runtime implementation;
- final DevKit CLI implementation;
- final complete block/item/recipe/entity schemas;
- final scripting language;
- final marketplace or trust UI;
- live hot-patching of running worlds;
- save/world migration format;
- renderer-internal ids, slots, atlas coordinates, or GPU handles.

Those remain follow-up implementation issues.

## Authoring package layout

A data-driven package can use this conceptual layout:

```text
mods/example.gems/
  mod.toml
  content/
    blocks/
      ruby_ore.toml
      ruby_block.toml
    items/
      ruby.toml
    recipes/
      ruby_block.toml
    materials/
      block_ruby_ore.toml
    models/
      block_ruby_ore.toml
    entities/
      firefly.toml
    behaviors/
      firefly_hover.toml
  assets/
    textures/
      block/
        ruby_ore.png
      item/
        ruby.png
```

Ownership:

- `mod.toml` is package identity, dependency, capability, and schema reference
  metadata;
- `content/` files are authored content source;
- `assets/` files are resource bytes;
- generated atlases/load plans are cache;
- save/world state is runtime persistence, not authored source.

A content pack can use the same `content/` and `assets/` layout without an
executable guest artifact.

## Friendly identity and shorthand

The canonical identity is always `namespace:path`.

The authoring layer may allow package-local shorthand only when tooling can expand
it deterministically.

Example package namespace:

```toml
id = "example.gems"
```

Friendly shorthand:

```toml
key = "blocks/ruby_ore"
drops = "items/ruby"
texture = "textures/block/ruby_ore"
```

Canonical expansion:

```text
example.gems:blocks/ruby_ore
example.gems:items/ruby
example.gems:textures/block/ruby_ore
```

Rules:

- canonical `namespace:path` is the stored semantic identity;
- shorthand is only a source convenience;
- diagnostics should show both source shorthand and expanded key;
- cross-namespace references must be explicit or imported through a declared
  dependency alias;
- generated runtime ids, block ids, palette slots, atlas coordinates, and cache
  paths are invalid in authoring files.

## Minimal block authoring

A minimal block file should be enough for a simple solid block.

Conceptual file:

```toml
schema = 1
key = "blocks/ruby_ore"
display_name = "Ruby Ore"

[block]
solid = true
opaque = true
hardness = 3.0

[visual]
texture = "textures/block/ruby_ore"
fallback_debug_tint_rgba = "C02040FF"

[drops]
item = "items/ruby"
min = 1
max = 3
```

The authoring compiler expands this into semantic content such as, following
[BLOCK_VISUAL_DEFINITIONS_v1.md](BLOCK_VISUAL_DEFINITIONS_v1.md):

- `add block:example.gems:blocks/ruby_ore`;
- `add material:example.gems:materials/block/ruby_ore` if the friendly block owns
  an implicit simple material;
- `add model:example.gems:models/block/ruby_ore` following
  [MODEL_ASSET_FORMAT_v1.md](MODEL_ASSET_FORMAT_v1.md), or reference a default
  cube model where the selected experience policy allows it;
- `add/patch visual:example.gems:visuals/block/ruby_ore`;
- `add/append tag` entries where tags are present;
- `add drops/loot` entry where the selected schema supports it.

The generated semantic operations must be explainable by DevKit.

## Minimal item authoring

A minimal item file should describe inventory-facing content.

Conceptual file:

```toml
schema = 1
key = "items/ruby"
display_name = "Ruby"

[item]
stack_size = 64

[visual]
texture = "textures/item/ruby"
fallback_debug_tint_rgba = "C02040FF"
```

The compiler expands this into semantic content such as:

- item entry;
- item visual binding;
- optional generated simple material/model references;
- dependencies on texture/material/model keys.

Item authoring should not imply gameplay behavior beyond declared fields.

## Minimal recipe authoring

A simple recipe should be readable without code.

Conceptual file:

```toml
schema = 1
key = "recipes/ruby_block"

[recipe]
kind = "crafting_shaped"
pattern = [
  "RRR",
  "RRR",
  "RRR",
]

[recipe.inputs]
R = "items/ruby"

[recipe.output]
item = "blocks/ruby_block"
count = 1
```

Rules:

- `kind` must be supported by the selected experience or stack;
- inputs and outputs must resolve to content keys;
- recipe keys are stable content entries;
- replacing or disabling an existing recipe must use explicit replace/disable
  semantics, not filename accidents;
- diagnostics should name the recipe file, symbol, missing key, and selected
  recipe kind.

## Minimal entity authoring

A simple entity file should describe data and reference behavior providers.

Conceptual file:

```toml
schema = 1
key = "entities/firefly"
display_name = "Firefly"

[entity]
tags = ["ambient"]

[visual]
model = "models/entity/firefly"
material = "materials/entity/firefly"

[components]
health = 2
collision = "small_flying"
ambient_light = 0.4

[behaviors]
controller = "behaviors/firefly_hover"
```

Rules:

- components are typed data;
- behavior references are namespaced keys after expansion;
- behavior implementations may be supplied by Wasm, future scripts, or
  experience-provided behavior families;
- missing behavior providers are validation errors;
- behavior-bearing entities may require server/experience approval.

## Behavior binding workflow

The authoring layer should let data-only content reference code-backed behavior.

Example behavior declaration:

```toml
schema = 1
key = "behaviors/firefly_hover"

[behavior]
provider = "example.gems.wasm:firefly_hover"
authority = "server"
```

A Wasm mod or future script pack can provide the behavior implementation through
public contracts. The content file only declares the binding.

Rules:

- data files do not get private engine APIs;
- behavior providers must be declared and validated;
- required capabilities must be visible in diagnostics;
- behavior config remains authored content/config data, not generated cache.

## Tags and families

Variant family expansion is defined by
[CONTENT_VARIANT_FAMILY_SCHEMA_v1.md](CONTENT_VARIANT_FAMILY_SCHEMA_v1.md).

Tags and families should be beginner-friendly but deterministic.

Conceptual tag file:

```toml
schema = 1
key = "tags/ores"

[tag]
members = [
  "blocks/ruby_ore",
]
```

Conceptual family file:

```toml
schema = 1
key = "families/ruby"

[family]
kind = "ore_set"

[family.members]
ore = "blocks/ruby_ore"
block = "blocks/ruby_block"
gem = "items/ruby"
```

Rules:

- tag members should be keyed or unique;
- duplicate members are diagnostics unless the schema defines idempotence;
- families should compile to explicit keyed relationships;
- order-sensitive collections need stable ordering rules;
- appending to existing tags/families uses content patch semantics.

## Explicit patch files

The authoring layer may provide explicit patch files for modifying existing
content.

Conceptual patch file:

```toml
schema = 1

[[patches]]
op = "patch"
kind = "block"
target = "freven.vanilla:blocks/stone"
path = "visual.material"
value = "example.pack:materials/block/stone_polished"
reason = "selected visual refresh"

[[patches]]
op = "disable"
kind = "recipe"
target = "freven.vanilla:recipes/stone_pickaxe"
reason = "progression pack disables this recipe"
```

Rules:

- explicit patch files compile directly to `CONTENT_PATCH_MERGE_v1` operations;
- patching another namespace requires dependency and policy;
- missing targets are errors unless the patch is explicitly optional;
- same-field conflicts must be reported deterministically;
- authoritative changes may require compatibility/migration policy.

## Asset references in friendly files

Friendly files can reference textures/materials/models with shorthand, but the
expanded result is still a stable visual asset key.

Example:

```toml
[visual]
texture = "textures/block/ruby_ore"
model = "models/block/cube_all"
material = "materials/block/ruby_ore"
```

Rules:

- texture shorthand expands to a visual asset key;
- asset files live under `assets/`;
- material/model declarations live under `content/`;
- generated atlas and texture-array ids are not valid authoring values;
- DevKit should explain the source file, expanded asset key, and missing asset
  file if resolution fails.

## Compilation pipeline

A data-driven authoring pipeline should be conceptually:

1. Discover selected package roots from the product/experience/stack.
2. Load manifests and active config references.
3. Load friendly content files under `content/`.
4. Expand shorthand ids to canonical namespaced keys.
5. Convert friendly files into semantic content operations.
6. Apply `CONTENT_PATCH_MERGE_v1` ordering and policy.
7. Resolve visual asset references through `VISUAL_ASSET_MODEL_v1`.
8. Apply asset override policy through `LAYERED_ASSET_OVERRIDES_v1`.
9. Validate authority, compatibility, dependencies, and references.
10. Build generated load plans/cache.
11. Start runtime with resolved public contracts only.

The compiler must be deterministic. It must not depend on filesystem traversal,
archive entry order, hash-map iteration order, or accidental discovery order.

## DevKit command expectations

The final CLI belongs to DevKit/Boot implementation, but the authoring layer
should support commands with these responsibilities:

```text
freven content check
freven content explain <key>
freven content expand <file>
freven content graph
freven content new block <key>
freven content new item <key>
freven content new recipe <key>
freven content new entity <key>
```

Expected behavior:

- `check` validates friendly files and semantic operations;
- `explain` shows where a key came from, what won, and what it depends on;
- `expand` shows generated semantic operations for a friendly file;
- `graph` shows content and visual dependency graph;
- `new` creates starter files with stable keys and comments.

The exact executable name and subcommand shape may differ. The SDK contract is
the diagnostic/authoring responsibility, not the CLI spelling.

## Diagnostics

Beginner diagnostics should be actionable.

Examples:

| Problem | Diagnostic should mention |
| --- | --- |
| Missing key | file path, field, expected `namespace:path` or expandable shorthand |
| Invalid shorthand | source value, package namespace, expected shape |
| Unknown field | file path, field path, allowed fields or extension map |
| Wrong type | field path, expected type, actual value |
| Missing texture file | source texture shorthand, expanded key, expected asset path |
| Missing material/model key | file path, expanded key, dependency owner |
| Unknown recipe item | recipe file, symbol, unresolved key |
| Missing behavior provider | behavior key, required package/capability |
| Forbidden cross-namespace patch | target owner, patch package, required policy |
| Duplicate content key | both source files and content kind |
| Generated cache referenced | invalid value and authored source category to use instead |

Diagnostics should point to the friendly file first, then show the semantic
operation that failed.

## Compatibility and authority

Authoring files must preserve authority boundaries.

Rules:

- authoritative blocks/items/recipes/entities participate in compatibility
  identity;
- client-local cosmetic authoring is allowed only where selected policy permits;
- behavior-bearing content may require server approval;
- save/world state is migrated separately;
- generated cache is never edited as authoring source;
- server-required visual/content bindings may be fingerprinted.

A resource-pack-style file must not silently become a gameplay patch.

## Relationship to Wasm

The authoring layer should compose with Wasm rather than replace it.

Data-only content is enough for simple content. Wasm remains the path for custom
runtime behavior, providers, actions, worldgen, controllers, and advanced
systems.

Rules:

- friendly content can reference Wasm-provided behavior keys;
- Wasm guests can consume resolved content through public SDK contracts;
- content files do not bypass capability declarations;
- missing Wasm/provider dependencies are validation errors;
- authors should be able to start data-only and later add code-backed behavior
  without changing the content identity model.

## Relationship to creator content schema

[CREATOR_CONTENT_SCHEMA_v1.md](CREATOR_CONTENT_SCHEMA_v1.md) defines the schema
direction and shared entry model.

This document defines the practical authoring workflow on top of that direction:
package layout, minimal files, shorthand expansion, compilation pipeline, DevKit
responsibilities, and beginner diagnostics.

If the friendly syntax and semantic model disagree, the semantic model wins.

## Relationship to implementation issues

This document is still docs/contract work. Follow-up implementation should happen
in SDK, DevKit, engine, and Vanilla as needed.

Likely follow-ups:

- SDK typed schema structs for friendly content files;
- DevKit validation and `content explain` commands;
- engine/boot resolver support for data-driven content;
- Vanilla migration to authored data files;
- sample data-only mod package;
- content pack packaging template;
- standalone product template using zero-Rust content.

## Acceptance checklist

A first implementation of this authoring layer should be able to demonstrate:

- a data-only block with texture/material/model references;
- a data-only item;
- a data-only recipe;
- a data-only entity with a behavior reference;
- a validation error for an unresolved reference;
- a validation error for invalid shorthand/key shape;
- an explanation of generated semantic operations;
- compatibility with a Wasm behavior provider;
- no renderer-internal ids in authored files;
- no generated cache edited as source.

## Non-goals

This document does not define:

- complete final schemas for every content kind;
- runtime content hot reload;
- marketplace packaging UI;
- save/world migration format;
- renderer internals;
- final CLI names;
- final scripting language;
- replacement of Wasm for advanced behavior.

Those remain separate implementation and design follow-ups.

## Relationship to lighting foundation

The practical authoring layer should expose lighting through the shared contract in
[LIGHTING_FOUNDATION_v1.md](LIGHTING_FOUNDATION_v1.md).

Rules:

- friendly lighting shorthand compiles into semantic content operations;
- DevKit diagnostics should explain unsupported lighting fields and deterministic
  fallbacks;
- generated lightmaps, probes, chunk light buffers, shader uniforms, renderer
  light handles, and lightmap coordinates are generated/backend output;
- lighting authoring must preserve the split between visual presentation and
  authoritative gameplay/world state.

## Relationship to shader/effect boundary

The practical authoring layer should expose shaders/effects through the shared
contract in [SHADER_EFFECT_BOUNDARY_v1.md](SHADER_EFFECT_BOUNDARY_v1.md).

Rules:

- friendly effect shorthand compiles into semantic effect declarations and
  references;
- DevKit validation should report missing effect keys, unsupported capabilities,
  denied raw source, and fallback choices;
- generated shader modules, bind group layouts, pipeline ids, GPU handles, and
  cache paths are generated/backend output;
- shader/effect authoring must preserve the split between visual presentation and
  authoritative gameplay/world state.
