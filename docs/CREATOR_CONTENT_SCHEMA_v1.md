# Creator Content Schema v1

This document defines the creator-friendly Freven content schema model.

It builds on:

- [ARCHITECTURE.md](ARCHITECTURE.md): engine / SDK / experience / mod /
  content-pack / standalone-product ownership;
- [PACKAGE_BOUNDARIES.md](PACKAGE_BOUNDARIES.md): manifest, config, content,
  assets, generated cache, and save/world state ownership;
- [VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md): visual asset identity,
  categories, dependency graph, validation, and renderer-backend boundaries;
- [LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md): visual asset
  layer ordering and override policy;
- [CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md): semantic add, replace,
  patch, append, disable, compatibility, and diagnostics model;
- [BLOCK_VISUAL_DEFINITIONS_v1.md](BLOCK_VISUAL_DEFINITIONS_v1.md): block visual
  bindings from gameplay block keys to model/material/tint/render policy.

The goal is to let simple Freven content be authored without Rust while keeping
the same long-term semantic model used by advanced Wasm mods, standalone games,
Vanilla extensions, and content packs.

## Core rule

Creator-friendly files are source syntax, not a second content model.

A simple block, item, recipe, material, model, entity, or behavior file compiles
into the same semantic content entries and patch/merge operations described by
[CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md).

This keeps beginner authoring simple without creating hidden engine-specific
shortcuts or Vanilla-only rules.

## Goals

- Make simple content authoring possible without Rust.
- Keep the beginner path readable and small.
- Keep advanced mods extensible through namespaced keys, components, behaviors,
  and Wasm/script integration points.
- Validate with useful errors that point to the exact file, field, key, and
  missing dependency.
- Keep Vanilla content out of engine core.
- Make zero-Vanilla standalone games first-class.
- Avoid renderer-internal ids, slots, atlas coordinates, or runtime object ids in
  author-facing data.

## Non-goals

This document does not define:

- final runtime implementation;
- final DevKit command behavior;
- final full block/item/recipe/entity/material/model schemas;
- final marketplace or trust UI;
- final save/world migration format;
- runtime hot-patching of live worlds;
- a replacement for Wasm mods;
- a second content composition model separate from content patch/merge semantics.

Those are follow-up implementation and authoring-layer issues.

## Schema layers

Creator content schema v1 has three conceptual layers.

| Layer | Purpose |
| --- | --- |
| Creator file syntax | Friendly TOML/YAML/JSON-like files for common content |
| Semantic content operations | Add/replace/patch/append/disable/test operations from `CONTENT_PATCH_MERGE_v1` |
| Runtime resolved graph | Validated effective content, asset graph, compatibility fingerprints, generated cache |

The first layer is for humans. The second layer is the canonical SDK semantics.
The third layer is host/runtime output.

## File families

A package may contain creator-friendly content files under `content/`.

Conceptual layout:

```text
content/
  blocks/
  items/
  recipes/
  entities/
  materials/
  models/
  visuals/
  tags/
  families/
  behaviors/
  components/
```

Each file family should map to one or more content kinds:

| File family | Typical content kinds |
| --- | --- |
| `content/blocks/` | block entries, block visual bindings, block tags, drops |
| `content/items/` | item entries, item visuals, inventory metadata |
| `content/recipes/` | recipe entries, crafting/smelting/processing definitions |
| `content/entities/` | entity archetypes, entity visuals, entity components |
| `content/materials/` | material declarations |
| `content/models/` | model declarations or model wrappers |
| `content/visuals/` | visual bindings from gameplay content to material/model keys |
| `content/tags/` | semantic tag membership |
| `content/families/` | variant families such as wood sets, stone sets, ore sets |
| `content/behaviors/` | behavior declarations bound to Wasm/script/native-capable systems |
| `content/components/` | reusable typed component declarations |

The exact file extensions and final schemas are follow-up work. This document
defines the shared shape and validation model.

## Shared entry shape

Every creator-facing entry should have a small common shape.

Conceptual example:

```toml
schema = 1
key = "example.gems:blocks/ruby_ore"
kind = "block"
display_name = "Ruby Ore"

[visual]
model = "example.gems:models/block/ruby_ore"
material = "example.gems:materials/block/ruby_ore"

[components]
solid = true
hardness = 3.0
drops = "example.gems:items/ruby"
```

Required common fields:

- `schema`;
- stable namespaced `key`;
- content kind, if the file family does not imply it;
- authored fields owned by that content kind.

Recommended common fields:

- display name or localization key;
- tags;
- visual references where relevant;
- component map;
- behavior references;
- explicit authority/compatibility hints where needed.

## Namespaced identity

Creator files use the same stable `namespace:path` identity as the rest of the
SDK.

Rules:

- every authored entry has one stable key;
- the namespace identifies the owning package/mod/experience;
- cross-namespace authoring requires explicit policy;
- shorthand ids may be allowed by tools only when they expand to stable
  namespaced keys;
- diagnostics should show both the shorthand and expanded key when applicable;
- runtime ids, renderer slots, atlas coordinates, and generated cache paths are
  never valid creator-facing identity.

Example shorthand expansion:

```text
blocks/ruby_ore
=> example.gems:blocks/ruby_ore
```

The full expanded key is the canonical identity.

## Blocks

A beginner block file should cover common block content without requiring code.

Conceptual block file:

```toml
schema = 1
key = "example.gems:blocks/ruby_ore"
display_name = "Ruby Ore"

[block]
solid = true
opaque = true
hardness = 3.0
tags = ["freven:stones", "example.gems:ores"]

[visual]
model = "example.gems:models/block/cube_all"
material = "example.gems:materials/block/ruby_ore"

[drops]
item = "example.gems:items/ruby"
min = 1
max = 3
```

A block file should compile into semantic content entries such as:

- block entry;
- block visual binding;
- tag membership;
- optional drops/loot entry;
- dependencies on materials, models, and items.

Block files must not define renderer slots or runtime block ids.

## Items

A beginner item file should describe inventory-facing content.

Conceptual item file:

```toml
schema = 1
key = "example.gems:items/ruby"
display_name = "Ruby"

[item]
stack_size = 64
tags = ["example.gems:gems"]

[visual]
model = "example.gems:models/item/ruby"
material = "example.gems:materials/item/ruby"
```

An item file should compile into semantic entries such as:

- item entry;
- item visual binding;
- tag membership;
- dependencies on material/model keys.

Item files should not imply behavior beyond the fields declared by the schema.

## Recipes

A recipe file should express common crafting/processing rules in data.

Conceptual recipe file:

```toml
schema = 1
key = "example.gems:recipes/ruby_block"
display_name = "Ruby Block"

[recipe]
kind = "crafting_shaped"
pattern = [
  "RRR",
  "RRR",
  "RRR",
]

[recipe.inputs]
R = "example.gems:items/ruby"

[recipe.output]
item = "example.gems:blocks/ruby_block"
count = 1
```

Recipe rules:

- input and output keys must resolve;
- recipe kind must be supported by the selected experience/stack;
- ambiguous shorthand should be expanded by tooling before validation;
- disabling/replacing recipes should use content patch/merge semantics, not
  hidden filename tricks.

## Materials

The canonical material schema is defined by [MATERIAL_DEFINITIONS_v1.md](MATERIAL_DEFINITIONS_v1.md).
Texture size, sampling, alpha, and validation policy are defined by
[TEXTURE_AUTHORING_v1.md](TEXTURE_AUTHORING_v1.md).

A beginner material file should reference textures and render-facing properties
without exposing renderer internals.

Conceptual material file:

```toml
schema = 1
key = "example.gems:materials/block/ruby_ore"

[material]
base_color_texture = "example.gems:textures/block/ruby_ore"
fallback_debug_tint_rgba = "C02040FF"
alpha_mode = "opaque"
render_layer = "solid"
```

Material files should compile into material content entries that reference visual
asset keys. They must not expose palette ids, atlas coordinates, texture-array
layers, GPU handles, or backend pipeline ids.

## Models and visuals

A model file describes reusable visual layout or geometry. A visual binding file
connects gameplay content to models/materials. Block visual binding details are
defined by [BLOCK_VISUAL_DEFINITIONS_v1.md](BLOCK_VISUAL_DEFINITIONS_v1.md).

Conceptual model file:

```toml
schema = 1
key = "example.gems:models/block/ruby_ore"

[model]
kind = "cube"
materials.all = "example.gems:materials/block/ruby_ore"
```

Conceptual visual binding:

```toml
schema = 1
key = "example.gems:visuals/block/ruby_ore"

[visual]
target = "example.gems:blocks/ruby_ore"
model = "example.gems:models/block/ruby_ore"
```

Model and visual files should reference author-facing asset keys, not host
renderer handles.

## Entities

An entity file should describe archetypes and components without requiring code
for simple cases.

Conceptual entity file:

```toml
schema = 1
key = "example.creatures:entities/firefly"
display_name = "Firefly"

[entity]
tags = ["example.creatures:ambient"]

[visual]
model = "example.creatures:models/entity/firefly"
material = "example.creatures:materials/entity/firefly"

[components]
health = 2
collision = "small_flying"
ambient_light = 0.4

[behaviors]
controller = "example.creatures:behaviors/firefly_hover"
```

Entity files should remain data declarations. Behavior execution belongs to
declared behavior systems, Wasm guests, scripts, or host-provided behavior
families.

## Behaviors and components

Creator-friendly behavior references bridge simple data to advanced execution.

Rules:

- components are typed data;
- behavior references are namespaced keys;
- behavior implementations may be provided by Wasm, future scripts, builtin
  experience systems, or safe host-provided behavior families;
- a behavior reference may require a declared capability;
- missing behavior providers are validation errors;
- behavior data must not bypass the public guest/runtime authority model.

Conceptual behavior binding:

```toml
[behaviors]
on_interact = "example.gems:behaviors/open_ruby_chest"

[behavior_config.open_ruby_chest]
loot_table = "example.gems:loot/ruby_chest"
```

This allows beginner content to connect to advanced code without making Rust a
requirement for every content file.

## Extension model

Creator schemas need an extension path for advanced mods.

Rules:

- standard fields are validated strictly;
- unknown top-level fields are rejected by default;
- extension fields must be namespaced or placed under an explicit extension map;
- extension owners must declare the schema/capability that validates them;
- extension data should compile into the same semantic content graph;
- extension data must not require engine internals.

Conceptual extension field:

```toml
[extensions."example.magic:mana"]
cost = 5
regen_delay = 2.0
```

This keeps the beginner schema clean while allowing advanced mods to add typed
data safely.

## Compilation to semantic operations

Creator files compile into content patch/merge operations.

Examples:

| Creator action | Semantic operation |
| --- | --- |
| New `content/blocks/ruby_ore.toml` | `add` block entry plus visual/tag/drop entries |
| New material file | `add` material entry |
| Recipe override file with explicit target | `replace` or `patch` recipe entry |
| Tag membership file | `append` keyed members |
| Hidden item flag | `patch` UI visibility field or `hide` operation |
| Disabled recipe file | `disable` recipe entry where schema allows it |

The compiler must be deterministic and diagnostics should mention both the
creator file and the resulting semantic operation.

## Validation model

Creator schema validation should happen before runtime start.

Validation should catch:

- missing `schema`;
- unsupported schema version;
- missing or invalid key;
- invalid `namespace:path`;
- unknown field;
- wrong field type;
- missing required field;
- invalid enum value;
- duplicate content entry;
- duplicate collection member;
- unresolved item/block/recipe/material/model/texture/behavior key;
- unsupported recipe kind;
- missing behavior provider or capability;
- forbidden cross-namespace patch;
- forbidden authoritative/cosmetic mismatch;
- renderer-internal id in authored data;
- save/world compatibility risk without policy;
- generated cache referenced as source.

Diagnostics should report:

- file path;
- line/field path where possible;
- content kind;
- key;
- owner package/layer;
- expected type or allowed values;
- referenced missing key;
- suggested fix.

## Compatibility with Wasm mods

Creator-friendly content and Wasm mods must compose through the same public
contracts.

Rules:

- data-only content can reference behavior keys implemented by Wasm systems;
- Wasm systems can consume resolved content through public SDK contracts;
- creator files do not get private engine APIs;
- creator files do not bypass capability declarations;
- advanced behavior remains opt-in through declared providers, actions,
  components, messages, or future scripting contracts;
- diagnostics should explain when a data file requires a missing Wasm/provider
  dependency.

This lets simple mods start as data and grow into code-backed mods later.

## Client/server and authority model

Creator schema must preserve authority boundaries.

Rules:

- authoritative content files participate in compatibility identity;
- client-local cosmetic files are allowed only where policy permits;
- behavior-bearing content may require server approval;
- save/world state is migrated separately;
- generated cache is never edited as creator source;
- server-required visual/content bindings can be fingerprinted.

A resource-pack-style cosmetic file should not silently become a gameplay patch.

## Relationship to content patch/merge

[CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md) defines canonical
composition semantics.

This document defines creator-friendly source shapes that compile into those
semantics.

If there is a disagreement, the semantic model wins. The friendly syntax should
be changed rather than creating a second interpretation.

## Relationship to authoring layer

This document defines schema direction. The practical authoring workflow is
defined in [DATA_DRIVEN_AUTHORING_LAYER_v1.md](DATA_DRIVEN_AUTHORING_LAYER_v1.md).

The authoring layer should later define:

- final file names and extensions;
- concrete schema versions;
- generated examples;
- DevKit validation commands;
- conversion from friendly files to semantic operations;
- packaging templates;
- beginner tutorials.

That implementation work should build on
[DATA_DRIVEN_AUTHORING_LAYER_v1.md](DATA_DRIVEN_AUTHORING_LAYER_v1.md).

## DevKit guidance

DevKit should eventually be able to:

- validate creator content files;
- explain how a file expands into semantic content entries and operations;
- show resolved references and missing dependencies;
- report unknown fields and wrong types with exact field paths;
- distinguish data-only content from code-backed behavior;
- show whether content is authoritative, server-required, cosmetic, or generated;
- suggest when a mod needs a Wasm behavior dependency;
- generate starter files for blocks, items, recipes, materials, models, and
  entities.

## Non-goals

This document does not define:

- final concrete syntax for every content kind;
- final block/item/entity runtime schema;
- final scripting language;
- final Wasm behavior API;
- final DevKit templates beyond the v1 authoring workflow;
- final implementation of content compilation;
- final save/world migration format;
- marketplace policy.

Those are separate follow-up issues.
