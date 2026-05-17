# Content Variant Family Schema v1

This document defines the Freven rc10 content variant/family expansion schema.

It builds on:

- [ARCHITECTURE.md](ARCHITECTURE.md): engine / SDK / experience / mod /
  content-pack / standalone-product ownership;
- [PACKAGE_BOUNDARIES.md](PACKAGE_BOUNDARIES.md): manifest, config, content,
  assets, generated cache, and save/world state ownership;
- [VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md): stable visual asset
  identity and renderer-internal boundaries;
- [MATERIAL_DEFINITIONS_v1.md](MATERIAL_DEFINITIONS_v1.md): material definitions
  that generated content may reference or create;
- [TEXTURE_AUTHORING_v1.md](TEXTURE_AUTHORING_v1.md): texture key and texture
  validation policy;
- [TEXTURE_BACKEND_PIPELINE_v1.md](TEXTURE_BACKEND_PIPELINE_v1.md): generated
  backend planning and internal atlas/array boundaries;
- [BLOCK_VISUAL_DEFINITIONS_v1.md](BLOCK_VISUAL_DEFINITIONS_v1.md): block visual
  bindings and variant hooks;
- [MODEL_ASSET_FORMAT_v1.md](MODEL_ASSET_FORMAT_v1.md): model asset declarations
  and variant hooks;
- [LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md): deterministic
  visual asset layering and override policy;
- [CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md): semantic add,
  replace, patch, append, disable, compatibility, and diagnostics model;
- [CREATOR_CONTENT_SCHEMA_v1.md](CREATOR_CONTENT_SCHEMA_v1.md): creator-facing
  source schema direction;
- [DATA_DRIVEN_AUTHORING_LAYER_v1.md](DATA_DRIVEN_AUTHORING_LAYER_v1.md):
  practical authoring workflow and shorthand expansion.

The goal is to let authors describe families such as soils, grass-covered soils,
wood species, stone types, ores, colored glass, plants, and crops without
copy-pasting every block, item, material, model, visual, tag, and drop entry by
hand.

## Core rule

Families are source-time expansion declarations.

A family describes how to generate normal semantic content entries from a compact
source declaration. It is not a runtime block id table, not a renderer variant
table, not save/world state, not a generated cache artifact, and not engine-side
hardcoded Vanilla logic.

Author-facing source:

~~~toml
schema = 1
key = "example.woods:families/planks"

[family]
kind = "content_family"

[[axes]]
name = "wood"
values = ["oak", "willow"]

[templates.block]
key = "blocks/{wood}_planks"
display_name = "{Wood} Planks"

[templates.visual]
model = "freven.core:models/block/cube_all"
material = "materials/block/{wood}_planks"
~~~

Generated semantic entries:

~~~text
block:example.woods:blocks/oak_planks
block:example.woods:blocks/willow_planks
visual:example.woods:visuals/block/oak_planks
visual:example.woods:visuals/block/willow_planks
material:example.woods:materials/block/oak_planks
material:example.woods:materials/block/willow_planks
~~~

Runtime/backend outputs that must stay internal:

~~~text
runtime block id
runtime item id
renderer material slot
atlas page and rectangle
texture array layer
GPU handle
generated cache path
save/world migration record
~~~

Only stable generated content keys are public author-facing output.

## Goals

- Define a content family concept with stable generated namespaced keys.
- Support variant axes such as soil type, grass coverage, stone type, glass color,
  wood species, ore type, plant stage, and crop stage.
- Support deterministic cartesian-product expansion.
- Support allow lists and skip lists for invalid combinations.
- Support per-variant and per-combination overrides.
- Support generated blocks, items, materials, models, visuals, tags, drops, loot,
  recipes, light/opacity metadata, collision/selection metadata, and tint hooks.
- Keep generated entries explicit, diagnosable, and compatible with normal
  content patch/merge semantics.
- Keep runtime ids, renderer slots, atlas coordinates, and generated cache out of
  authored family data.
- Provide a clean SDK target for DevKit validation and later implementation.

## Non-goals

This document does not define:

- final engine/runtime implementation;
- final DevKit command names;
- final complete block/item/material/model/visual schemas;
- final collision/selection schema;
- final recipe/loot schema;
- final tint/color-map pipeline;
- final save/world migration format;
- live runtime expansion after a world is already running;
- procedural generation language;
- marketplace or trust UI;
- Vanilla's actual family library.

Those are separate SDK, engine, DevKit, Vanilla, and product issues.

## Terminology

| Term | Meaning |
| --- | --- |
| Family | Source declaration that expands into semantic content entries. |
| Axis | Named variant dimension such as `wood`, `soil`, `color`, or `grass`. |
| Axis value | One value in an axis, such as `oak`, `sand`, `red`, or `covered`. |
| Combination | One selected value from each axis. |
| Template | Source block that creates generated entries for each combination. |
| Generated entry | Normal semantic content entry produced by family expansion. |
| Generated key | Stable namespaced key of a generated entry. |
| Allow list | Explicit list of combinations to generate. |
| Skip list | Explicit list of combinations to suppress. |
| Override | Per-value or per-combination patch applied during expansion. |
| Finalization | Build/load phase where family source expands into content operations. |
| Runtime id | Host/runtime compact id assigned after content resolution. Not authored. |

## Family keys

A family has a stable namespaced content key:

~~~text
namespace:families/name
~~~

Examples:

~~~text
freven.vanilla:families/soil
freven.vanilla:families/grass_soil
freven.vanilla:families/planks
freven.vanilla:families/colored_glass
example.gems:families/ores
example.crops:families/wheat
~~~

Rules:

- family keys are content keys;
- generated entries have their own stable keys;
- generated keys must be deterministic from source family data;
- generated keys must not encode runtime ids, renderer slots, atlas coordinates,
  texture-array layers, GPU handles, or cache paths;
- family declarations participate in content patch/merge semantics like other
  content entries.

## Ownership and location

Family declarations live in content data, normally under:

~~~text
content/families/
~~~

Example package layout:

~~~text
mods/example.woods/
  mod.toml
  content/
    families/
      planks.toml
      logs.toml
    materials/
      block/
        oak_planks.toml
        willow_planks.toml
    models/
      block/
        log_axis.toml
  assets/
    textures/
      block/
        oak_planks.png
        willow_planks.png
~~~

A creator-friendly block file may reference a family or embed a small family
source section, but the resolved result must still be explicit generated content
entries.

## Family schema v1

Canonical family files use:

~~~toml
schema = 1
key = "example.woods:families/planks"

[family]
kind = "content_family"
namespace = "example.woods"
description = "Wood plank blocks, items, materials, and visuals."

[[axes]]
name = "wood"
values = ["oak", "willow"]

[templates.block]
key = "blocks/{wood}_planks"
display_name = "{Wood} Planks"

[templates.visual]
key = "visuals/block/{wood}_planks"
target = "blocks/{wood}_planks"
model = "freven.core:models/block/cube_all"
material = "materials/block/{wood}_planks"
fallback_debug_tint_rgba = "{wood.fallback_tint}"
~~~

### Required fields

| Field | Type | Meaning |
| --- | --- | --- |
| `schema` | integer | Family file schema version. v1 uses `1`. |
| `key` | `namespace:path` | Stable family identity. |
| `family.kind` | enum | v1 uses `content_family`. |
| `axes` | array | Variant axes. |
| `templates` | map | Generated content templates. |

### Common fields

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `family.namespace` | namespace | family key namespace | Namespace for package-local generated keys. |
| `family.description` | string | none | Diagnostic/help text. |
| `family.key_pattern` | string | template-specific | Optional shared generated key pattern. |
| `family.display_name_pattern` | string | template-specific | Optional shared display-name pattern. |
| `allow` | list | all combinations | Explicit combinations to include. |
| `skip` | list | none | Explicit combinations to exclude. |
| `overrides` | array/table | none | Per-value or per-combination overrides. |
| `templates` | map | required | Content templates to generate. |
| `diagnostics` | map | default policy | Optional strictness and report hints. |

## Axes

An axis is a named dimension of variation.

~~~toml
[[axes]]
name = "wood"
values = ["oak", "willow", "pine"]
~~~

An axis value may be a plain string or a structured object.

~~~toml
[[axes]]
name = "color"

[[axes.values]]
id = "red"
display = "Red"
fallback_tint_rgba = "CC3030FF"
texture_suffix = "red"

[[axes.values]]
id = "blue"
display = "Blue"
fallback_tint_rgba = "3050CCFF"
texture_suffix = "blue"
~~~

Rules:

- axis names must be stable inside the family;
- axis value ids must be stable;
- axis and value ids should use lowercase resource-key-friendly names;
- display labels are authoring metadata, not generated identity;
- axis value metadata may be referenced by templates;
- duplicate axis names or duplicate value ids are validation errors.

Recommended axis examples:

| Axis | Example values |
| --- | --- |
| `soil` | `dirt`, `sand`, `clay`, `peat` |
| `grass` | `bare`, `sparse`, `covered` |
| `stone` | `granite`, `limestone`, `basalt` |
| `color` | `white`, `red`, `blue`, `green` |
| `wood` | `oak`, `willow`, `pine` |
| `ore` | `ruby`, `copper`, `tin` |
| `growth_stage` | `stage0`, `stage1`, `stage2`, `mature` |

## Combination expansion

By default, a family expands the cartesian product of all axis values.

~~~toml
[[axes]]
name = "soil"
values = ["dirt", "sand"]

[[axes]]
name = "grass"
values = ["bare", "covered"]
~~~

Generated combinations:

~~~text
soil=dirt, grass=bare
soil=dirt, grass=covered
soil=sand, grass=bare
soil=sand, grass=covered
~~~

Rules:

- expansion order must be deterministic;
- axis order is the order declared in source unless an explicit order is given;
- value order is the order declared in source unless an explicit order is given;
- generated diagnostics should use this deterministic order;
- the same source family must produce the same generated entries and same
  generated-key list across machines.

Recommended deterministic sort key:

~~~text
family_key
axis_declaration_order
axis_value_declaration_order
template_kind_order
generated_key
~~~

## Generated keys

Generated keys come from templates.

~~~toml
[templates.block]
key = "blocks/{wood}_planks"

[templates.item]
key = "items/{wood}_planks"

[templates.material]
key = "materials/block/{wood}_planks"
~~~

For package-local shorthand, the family namespace is applied:

~~~text
example.woods:blocks/oak_planks
example.woods:items/oak_planks
example.woods:materials/block/oak_planks
~~~

Rules:

- canonical generated keys are `namespace:path`;
- shorthand is a source convenience only;
- generated keys must be visible in diagnostics;
- generated keys must be stable unless the family source intentionally changes;
- generated keys must not depend on filesystem order, hash-map order, runtime ids,
  renderer state, atlas packing, generated cache, or previous build output;
- key collisions are validation errors unless resolved by explicit patch/replace
  semantics.

## Template substitution

Templates may reference axis values and axis metadata.

Supported conceptual substitutions:

| Expression | Meaning |
| --- | --- |
| `{wood}` | Axis value id. |
| `{Wood}` | Title-cased display form. |
| `{wood.display}` | Structured axis value display string. |
| `{wood.texture_suffix}` | Structured axis value metadata field. |
| `{color.fallback_tint_rgba}` | Structured axis value metadata field. |
| `{combination.key}` | Deterministic joined combination key, if defined by policy. |

Example:

~~~toml
[templates.material]
key = "materials/block/{wood}_planks"
base_color_texture = "textures/block/{wood.texture_suffix}_planks"
fallback_debug_tint_rgba = "{wood.fallback_tint_rgba}"
~~~

Rules:

- unresolved substitution variables are validation errors;
- substitution must not execute code;
- substitution must not read filesystem state;
- substitution must not depend on runtime world state;
- all generated values should remain explainable by DevKit.

## Template kinds

A family may generate multiple content kinds.

Recommended v1 template keys:

| Template | Generated kind |
| --- | --- |
| `templates.block` | block content entry |
| `templates.item` | item content entry |
| `templates.material` | material content entry |
| `templates.model` | model content entry |
| `templates.visual` | block/item/entity visual binding |
| `templates.tags` | tag membership entries |
| `templates.drops` | drop/loot entries |
| `templates.recipe` | recipe entries |
| `templates.light` | light/opacity metadata |
| `templates.collision` | collision metadata |
| `templates.selection` | selection metadata |

Rules:

- generated entries are normal semantic content entries;
- generated entries must validate against their owning schema;
- family expansion does not create a second content model;
- unsupported template kinds must produce diagnostics;
- the selected experience/product may restrict which template kinds are allowed.

## Wood family example

~~~toml
schema = 1
key = "example.woods:families/planks"

[family]
kind = "content_family"

[[axes]]
name = "wood"

[[axes.values]]
id = "oak"
display = "Oak"
fallback_tint_rgba = "B8864BFF"

[[axes.values]]
id = "willow"
display = "Willow"
fallback_tint_rgba = "C5B174FF"

[templates.block]
key = "blocks/{wood}_planks"
display_name = "{wood.display} Planks"
solid = true
opaque = true
hardness = 2.0

[templates.item]
key = "items/{wood}_planks"
display_name = "{wood.display} Planks"

[templates.material]
key = "materials/block/{wood}_planks"
base_color_texture = "textures/block/{wood}_planks"
fallback_debug_tint_rgba = "{wood.fallback_tint_rgba}"
render_layer = "opaque"

[templates.visual]
key = "visuals/block/{wood}_planks"
target = "blocks/{wood}_planks"
model = "freven.core:models/block/cube_all"
material = "materials/block/{wood}_planks"
fallback_debug_tint_rgba = "{wood.fallback_tint_rgba}"

[[templates.tags]]
tag = "tags/blocks/planks"
value = "blocks/{wood}_planks"
~~~

This generates block, item, material, visual, and tag entries for each wood value.

## Soil and grass family example

~~~toml
schema = 1
key = "freven.vanilla:families/grass_soil"

[family]
kind = "content_family"

[[axes]]
name = "soil"
values = ["dirt", "sand", "clay"]

[[axes]]
name = "grass"
values = ["bare", "covered"]

[[skip]]
soil = "sand"
grass = "covered"
reason = "grass-covered sand is not part of this experience."

[templates.block]
key = "blocks/{grass}_{soil}"
display_name = "{Grass} {Soil}"
solid = true
hardness = 0.6

[templates.materials.top]
key = "materials/block/{grass}_{soil}_top"
base_color_texture = "textures/block/{grass}_{soil}_top"
fallback_debug_tint_rgba = "6BAA3AFF"

[templates.materials.side]
key = "materials/block/{grass}_{soil}_side"
base_color_texture = "textures/block/{grass}_{soil}_side"
fallback_debug_tint_rgba = "6BAA3AFF"

[templates.materials.bottom]
key = "materials/block/{soil}"
base_color_texture = "textures/block/{soil}"
fallback_debug_tint_rgba = "8A5A30FF"

[templates.visual]
key = "visuals/block/{grass}_{soil}"
target = "blocks/{grass}_{soil}"
model = "freven.core:models/block/cube_faces"
fallback_debug_tint_rgba = "6BAA3AFF"

[templates.visual.materials]
top = "materials/block/{grass}_{soil}_top"
side = "materials/block/{grass}_{soil}_side"
bottom = "materials/block/{soil}"
~~~

Rules illustrated:

- invalid combinations are skipped explicitly;
- per-face visuals are generated through normal visual/model/material schemas;
- generated block keys stay stable and visible;
- runtime block ids are not authored.

## Colored glass family example

~~~toml
schema = 1
key = "freven.vanilla:families/colored_glass"

[family]
kind = "content_family"

[[axes]]
name = "color"

[[axes.values]]
id = "red"
display = "Red"
fallback_tint_rgba = "CC3030AA"

[[axes.values]]
id = "blue"
display = "Blue"
fallback_tint_rgba = "3050CCAA"

[templates.block]
key = "blocks/{color}_glass"
display_name = "{color.display} Glass"
solid = true
opaque = false
hardness = 0.3

[templates.material]
key = "materials/block/{color}_glass"
base_color_texture = "textures/block/glass"
alpha_mode = "blend"
render_layer = "transparent"
fallback_debug_tint_rgba = "{color.fallback_tint_rgba}"

[templates.visual]
key = "visuals/block/{color}_glass"
target = "blocks/{color}_glass"
model = "freven.core:models/block/cube_all"
material = "materials/block/{color}_glass"
tint_rgba = "{color.fallback_tint_rgba}"
render_layer = "transparent"
fallback_debug_tint_rgba = "{color.fallback_tint_rgba}"
~~~

This keeps the color axis in authored content while renderer tint/material slots
remain internal generated state.

## Allow and skip lists

Use `allow` when only a small subset of combinations should exist.

~~~toml
[[allow]]
wood = "oak"
part = "planks"

[[allow]]
wood = "oak"
part = "log"

[[allow]]
wood = "willow"
part = "planks"
~~~

Use `skip` when most combinations should exist but some are invalid.

~~~toml
[[skip]]
soil = "sand"
grass = "covered"
reason = "No grass-covered sand in this experience."
~~~

Rules:

- `allow` is applied before `skip`;
- if `allow` is present, only matching combinations are candidates;
- `skip` removes matching candidates;
- overlapping or contradictory filters should produce diagnostics;
- filters should explain invalid combinations where helpful;
- diagnostics should report the family key, axis values, and reason.

## Overrides

Overrides apply source-level patches during family expansion.

Per-axis-value override:

~~~toml
[overrides.wood.oak.block]
hardness = 2.5
~~~

Per-combination override:

~~~toml
[[overrides.combination]]
when = { soil = "clay", grass = "bare" }

[overrides.combination.block]
hardness = 0.8

[overrides.combination.visual]
fallback_debug_tint_rgba = "7A5A40FF"
~~~

Rules:

- overrides apply after template substitution;
- override order must be deterministic;
- combination-specific overrides beat axis-value overrides;
- conflicts between overrides are diagnostics unless an explicit precedence rule
  exists;
- overrides must modify generated source fields, not runtime ids or renderer
  internals;
- overrides must validate against the target generated content schema.

Recommended precedence:

1. Base template.
2. Axis-value overrides in axis declaration order.
3. Combination overrides in source order.
4. Explicit content patch/merge operations after family expansion.

## Generated content operations

Family expansion produces normal semantic content operations.

Conceptual expansion output:

~~~text
add block:example.woods:blocks/oak_planks
add item:example.woods:items/oak_planks
add material:example.woods:materials/block/oak_planks
add visual:example.woods:visuals/block/oak_planks
append tag:example.woods:tags/blocks/planks -> example.woods:blocks/oak_planks
~~~

Rules:

- generated entries enter the same content graph as hand-written entries;
- generated entries can be patched, replaced, appended, or disabled through
  normal content patch/merge semantics;
- generated entries must include provenance back to the family key and source
  combination;
- duplicate generated keys are validation errors unless explicit policy resolves
  them;
- generated entries must be ordered deterministically for diagnostics and
  fingerprints.

## Tags, drops, recipes, and loot

Families may generate non-visual content.

Tag example:

~~~toml
[[templates.tags]]
tag = "tags/blocks/logs"
value = "blocks/{wood}_log"
~~~

Drop example:

~~~toml
[templates.drops]
target = "blocks/{ore}_ore"
item = "items/raw_{ore}"
min = 1
max = 3
~~~

Recipe example:

~~~toml
[templates.recipe]
key = "recipes/{wood}_planks_from_log"
kind = "crafting_shapeless"
inputs = ["blocks/{wood}_log"]
output = { item = "blocks/{wood}_planks", count = 4 }
~~~

Rules:

- generated non-visual entries are still normal semantic content;
- gameplay-affecting generated entries may be authoritative;
- cosmetic packs must not use visual-only policy to change generated drops,
  recipes, collision, selection, or save/world meaning;
- generated recipe/loot syntax belongs to the owning recipe/loot schema.

## Collision, selection, light, opacity, and tint

Families may affect gameplay/interaction metadata only through generated content
fields owned by the relevant schemas.

Conceptual example:

~~~toml
[templates.block]
key = "blocks/{color}_glass"
opaque = false
light_opacity = 1

[templates.collision]
target = "blocks/{color}_glass"
kind = "full_block"

[templates.selection]
target = "blocks/{color}_glass"
kind = "full_block"

[templates.visual]
tint_rgba = "{color.fallback_tint_rgba}"
tint_source = "constant"
~~~

Rules:

- collision and selection are gameplay/interaction content, not visual geometry;
- light and opacity metadata affect world/render/gameplay policy as defined by
  the selected experience;
- tint/color-map source is visual/content metadata, not renderer slot state;
- changing these fields through family expansion may affect compatibility;
- client-local cosmetic packs may not use family overrides to change gameplay
  collision, selection, drops, recipes, or save/world state.

## Patching families

Families are content entries and can be patched.

Example: add a wood type.

~~~toml
[[content_patches]]
op = "append"
kind = "family"
target = "example.woods:families/planks"
path = "axes.wood.values"
value = { id = "pine", display = "Pine", fallback_tint_rgba = "D0A060FF" }
authority = "selected_stack"
reason = "selected pack adds pine wood"
~~~

Example: skip one combination.

~~~toml
[[content_patches]]
op = "append"
kind = "family"
target = "freven.vanilla:families/grass_soil"
path = "skip"
value = { soil = "clay", grass = "covered", reason = "not supported by this pack" }
authority = "selected_stack"
reason = "selected pack narrows generated soil variants"
~~~

Rules:

- family patches apply before expansion finalization;
- patches to generated entries apply after expansion finalization;
- diagnostics must make this ordering clear;
- patching a family may create, remove, or change many generated entries;
- tools should show the generated-entry delta before accepting large changes.

## Disabling generated entries

Generated entries can be disabled explicitly.

Disable at family level:

~~~toml
[[skip]]
wood = "willow"
reason = "Willow is disabled in this experience."
~~~

Disable generated entry after expansion:

~~~toml
[[content_patches]]
op = "disable"
kind = "block"
target = "example.woods:blocks/willow_planks"
authority = "selected_stack"
reason = "Willow planks are disabled in this product."
~~~

Rules:

- family-level skip prevents generation;
- content-level disable acts on a generated semantic entry;
- both paths must be diagnosable;
- disabling gameplay-affecting generated entries may affect compatibility.

## Authority and compatibility

Families can affect compatibility because they generate real content.

| Class | Meaning |
| --- | --- |
| Cosmetic family | Generates only visual/cosmetic content allowed by policy. |
| Selected-stack family | Participates in selected experience/stack content identity. |
| Server-required family | Server requires the effective generated content graph or accepted equivalent. |
| Authoritative family | Generates gameplay/save/world-affecting content. |

Rules:

- family source may participate in compatibility fingerprints;
- generated entries may participate in compatibility fingerprints;
- authoritative generated entries affect save/world compatibility;
- cosmetic family changes must not modify gameplay semantics;
- diagnostics should show whether a family is cosmetic, selected-stack,
  server-required, or authoritative.

## Determinism and fingerprints

Family expansion must be deterministic.

Deterministic inputs:

- family key;
- selected package/layer stack;
- effective family declaration after patches;
- axis order and value order;
- allow/skip filters;
- template source;
- override source;
- selected content schema versions;
- selected experience/product policy.

Non-inputs:

- filesystem traversal order;
- hash-map iteration order;
- runtime block ids;
- renderer material slots;
- atlas packing;
- texture-array layers;
- GPU handles;
- generated cache paths;
- previous run output order.

A family fingerprint should include enough source data to detect generated content
drift. Generated-entry fingerprints should include provenance to the family key
and combination.

## Diagnostics

Family diagnostics should be first-class.

Diagnostics should report:

- family file path;
- family key;
- schema version;
- axis name;
- axis value id;
- combination values;
- template kind;
- generated content kind;
- generated key;
- override source;
- allow/skip source;
- selected package/layer;
- exact field path where possible;
- expected type or allowed values;
- suggested fix.

Required diagnostic cases:

- missing `schema`;
- unsupported schema version;
- missing or invalid family key;
- duplicate axis name;
- duplicate axis value id;
- empty axis values;
- invalid allow/skip axis;
- allow/skip references unknown value;
- all combinations skipped unintentionally;
- unresolved substitution variable;
- duplicate generated key;
- generated key collides with hand-written content;
- generated entry fails its owning schema validation;
- override targets unknown field;
- override changes renderer-internal/runtime/generated-cache field;
- generated authoritative content changed by cosmetic-only package;
- runtime id used in authored family data.

## DevKit guidance

DevKit should eventually be able to:

- validate family files before runtime start;
- list axes and values;
- preview generated combinations;
- show allow/skip filtering results;
- show generated keys by content kind;
- show generated block/material/model/visual/tag/drop/recipe entries;
- explain per-axis and per-combination overrides;
- report generated key collisions;
- show generated-entry provenance;
- show compatibility/fingerprint impact;
- diff generated output before and after a family patch;
- export resolved generated content for inspection;
- keep runtime ids, renderer slots, atlas coordinates, and generated cache paths
  out of author-facing family files.

## Relationship to block visuals

[BLOCK_VISUAL_DEFINITIONS_v1.md](BLOCK_VISUAL_DEFINITIONS_v1.md) defines block
visual entries and variant hooks.

This document defines how family expansion may generate those block visual
entries and how `visual.variant_selector`-style source hooks are finalized into
explicit generated visual entries.

If there is disagreement:

- family expansion owns generated entry creation;
- block visuals own block-to-model/material/tint/render binding;
- materials own surface properties;
- model format owns geometry/layout/material-slot declarations;
- engine meshing owns generated renderer mesh output.

## Relationship to model format

[MODEL_ASSET_FORMAT_v1.md](MODEL_ASSET_FORMAT_v1.md) defines model asset entries
and variant hooks.

This document defines how family expansion may generate model entries or bind
generated visuals to generated/static model keys.

If there is disagreement:

- family expansion owns generated model entry creation and generated-key policy;
- model format owns the shape and validation of each resulting model entry.

## Relationship to content patch/merge

Family expansion works with content patch/merge in two phases:

1. Compose and patch family declarations.
2. Expand families into semantic content entries.
3. Apply patches that target generated entries.

This ordering lets packs add values to a family, skip combinations, and patch
generated entries explicitly.

## Relationship to engine runtime

The engine/runtime consumes effective generated content.

The engine must not:

- hardcode Vanilla family axes;
- infer content families from renderer slots;
- use runtime block ids as authoring keys;
- use atlas coordinates or texture-array layers as family data;
- expand families nondeterministically during rendering.

The engine may assign runtime ids after content finalization. Those ids are
internal runtime state.

## Acceptance checklist

A correct v1 implementation/design should allow:

- a wood/planks family without hand-writing every block and item;
- a soil/grass family with invalid combinations skipped;
- a colored glass family with material/tint/render-layer variation;
- generated block/material/model/visual keys visible in diagnostics;
- generated tags/drops/recipes where selected schemas support them;
- per-variant overrides without renderer internals;
- deterministic output independent of filesystem/hash-map order;
- runtime block ids remaining internal.

## Summary

Content variant/family schema v1 defines deterministic source-time expansion from
compact family declarations into normal semantic content entries.

It lets authors describe large content sets such as woods, soils, grass variants,
stone types, ores, and colored glass while keeping generated keys stable,
diagnostics explicit, runtime ids internal, and renderer/backend state out of
authored data.

## Relationship to lighting foundation

Variant families may generate light, opacity, transmission, emissive, shading, and
AO metadata using the shared vocabulary in
[LIGHTING_FOUNDATION_v1.md](LIGHTING_FOUNDATION_v1.md).

Rules:

- generated lighting fields are normal semantic content entries;
- generated light/opacity metadata must remain deterministic and diagnosable;
- generated renderer light ids, lightmap coordinates, shader constants, and cache
  paths are never authored family output;
- Vanilla-like families such as torches, lamps, glass, leaves, water, and glowing
  plants should express lighting through content metadata, not engine hardcode.

## Conformance fixtures

Canonical family examples live under `fixtures/visual_content_schema_v1/valid/families/`.

The fixture set covers rock, soil/grass, and colored-glass families. Family
expansion is source/load-time content expansion that happens before resolved
visual load plans, client visual mesh table installation, and runtime meshing.
