# Content Patch and Merge Semantics v1

This document defines Freven's data-driven content add, replace, patch, append,
disable, and conflict semantics.

It builds on:

- [ARCHITECTURE.md](ARCHITECTURE.md): engine / SDK / experience / mod /
  content-pack / standalone-product ownership;
- [PACKAGE_BOUNDARIES.md](PACKAGE_BOUNDARIES.md): manifest, config, content,
  assets, generated cache, and save/world state ownership;
- [VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md): visual asset identity,
  categories, dependency graph, validation, and renderer-backend boundaries;
- [LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md): visual asset
  layer ordering and override policy.

The goal is to make authored content composition explicit, deterministic,
diagnosable, and safe for Vanilla extensions, mods, content packs, total
conversions, servers, and zero-Vanilla standalone games.

## Core rule

Content composition is resolved before runtime starts.

Content patches modify authored content definitions in the selected experience /
stack / package graph. They do not mutate save/world state, generated cache,
renderer handles, or live runtime state.

A patch must say what it targets, what operation it performs, and what authority
class it affects. Accidental duplicates are errors.

## Terminology

| Term | Meaning |
| --- | --- |
| Content entry | One authored data definition with a stable key and kind |
| Content key | Stable author-facing `namespace:path` identity |
| Content kind | Type of entry, such as block, item, recipe, material, model, visual binding, family, tag |
| Base entry | Existing entry being patched or replaced |
| Patch operation | Explicit add, replace, patch, append, remove, disable, hide, or test operation |
| Effective entry | Final entry after all selected layers and patches are applied |
| Tombstone | Explicit disabled/removed marker where the schema allows it |
| Conflict | Composition cannot produce one deterministic effective entry |
| Authoritative content | Content that affects server/world compatibility or gameplay meaning |
| Cosmetic content | Content that affects presentation only where policy allows |

## Content entry identity

A content entry is identified by at least:

- content kind;
- stable namespaced key;
- owner package/layer;
- schema version or content model version where relevant.

Examples:

```text
block:freven.vanilla:blocks/stone
item:freven.vanilla:items/stone
recipe:freven.vanilla:recipes/stone_pickaxe
material:freven.vanilla:materials/block/stone
model:freven.vanilla:models/block/cube_all
visual:freven.vanilla:visuals/block/stone
tag:freven.vanilla:tags/stone
family:freven.vanilla:families/planks
```

The exact file layout and schema syntax are follow-up documents. The semantic
requirement is stable identity plus explicit content kind.

## Operation model

Content patch/merge v1 recognizes these operations.

| Operation | Meaning |
| --- | --- |
| Add | Define a new entry that does not already exist in the effective graph |
| Replace | Replace an existing whole entry with a new whole entry |
| Patch | Modify selected fields in an existing entry |
| Append | Add keyed members to an ordered or unordered collection where schema allows it |
| Remove | Remove a field or keyed collection member where schema allows it |
| Disable | Mark an entry unavailable through an explicit tombstone where schema allows it |
| Hide | Hide from UI/creative lists where schema allows it without deleting compatibility identity |
| Test | Require a precondition before applying another operation |
| Conflict | Fail because the operation is ambiguous, forbidden, or incompatible |

All operations are explicit. A second `add` of an existing key is not an implicit
replace. A field edit is not an implicit deep merge unless the schema says that
field is patchable.

## Conceptual patch declaration

The final file syntax is a follow-up implementation detail. The semantic model
should support declarations shaped like this:

```toml
[[content_patches]]
op = "patch"
kind = "block"
target = "freven.vanilla:blocks/stone"
path = "visual.material"
value = "example.pack:materials/block/stone_polished"
authority = "server_required"
reason = "selected stack visual refresh"
```

Important fields:

- operation;
- content kind;
- target key;
- field path or keyed collection path when relevant;
- value or replacement;
- authority class;
- owner package/layer;
- optional preconditions;
- diagnostic reason/context.

A future beginner-friendly schema may hide this raw representation, but the
underlying semantics should remain explicit and inspectable.

## Add

`add` creates a new content entry.

Rules:

- the target key must not already exist in the effective graph;
- the declaring package normally owns its namespace;
- cross-namespace add is invalid unless a future namespace policy explicitly
  permits it;
- the new entry must validate against its content kind schema;
- dependencies referenced by the new entry must resolve;
- authoritative additions may affect server/world compatibility.

Example:

```toml
[[content_patches]]
op = "add"
kind = "block"
key = "example.gems:blocks/ruby_ore"
```

If another package already declared `example.gems:blocks/ruby_ore`, this is a
duplicate-key conflict unless there is an explicit replace/override relationship.

## Replace

`replace` swaps an existing whole content entry for a new whole entry.

Rules:

- the target must exist;
- replacement must have the same content kind unless a schema explicitly supports
  kind migration;
- cross-namespace replacement requires explicit policy;
- authoritative replacement may require compatibility/fingerprint updates;
- replacement should be preferred over many field patches when the author intends
  a full definition takeover.

Example:

```toml
[[content_patches]]
op = "replace"
kind = "material"
target = "freven.vanilla:materials/block/stone"
replacement = "example.total:materials/block/stone"
authority = "selected_stack"
```

This is different from visual asset override. Asset override decides which asset
key wins. Content replace changes a structured content entry.

## Patch

`patch` modifies selected fields in an existing entry.

Rules:

- the target must exist unless the operation is explicitly optional;
- the field path must exist or be creatable by schema;
- the value must type-check against the field schema;
- patchable fields must be declared by the content kind schema;
- patch order must be deterministic;
- conflicting writes to the same field require explicit policy or fail.

Example:

```toml
[[content_patches]]
op = "patch"
kind = "block"
target = "freven.vanilla:blocks/stone"
path = "visual.material"
value = "example.pack:materials/block/stone_polished"
```

A patch must not write renderer-internal slots, atlas coordinates, runtime ids, or
generated cache paths.

## Append

`append` adds members to a schema-approved collection.

Rules:

- append is allowed only for fields declared appendable;
- compatibility-sensitive arrays should use keyed members instead of fragile
  numeric indexes;
- duplicate keyed members are conflicts unless schema defines idempotent behavior;
- order-sensitive collections must define stable ordering rules;
- append is not a blanket deep merge.

Examples:

```toml
[[content_patches]]
op = "append"
kind = "family"
target = "freven.vanilla:families/planks"
path = "variants"
key = "example.woods:blocks/willow_planks"
```

```toml
[[content_patches]]
op = "append"
kind = "tag"
target = "freven.vanilla:tags/stone"
path = "members"
key = "example.gems:blocks/ruby_ore"
```

The schema decides whether a collection is ordered, unordered, keyed, unique, or
append-only.

## Remove, disable, and hide

Removal-like operations must be explicit and schema-approved.

| Operation | Use |
| --- | --- |
| Remove | Delete a field or keyed collection member where schema allows it |
| Disable | Make an entry unavailable through a tombstone while preserving identity/history |
| Hide | Hide from UI/creative/search surfaces without deleting gameplay compatibility identity |

Rules:

- authoritative remove/disable may require compatibility or migration handling;
- removing a key that other content depends on is a diagnostic unless schema
  defines fallback behavior;
- disabling content is different from deleting save/world state that already
  references it;
- hide is presentation policy, not gameplay removal.

Example:

```toml
[[content_patches]]
op = "disable"
kind = "recipe"
target = "freven.vanilla:recipes/stone_pickaxe"
authority = "server_required"
reason = "selected progression pack disables this recipe"
```

Save/world migrations are separate. A content tombstone may explain how old saves
should interpret a removed entry, but it is not itself a save migration.

## Test and preconditions

`test` or equivalent preconditions make patches safer.

Examples of useful preconditions:

- target exists;
- target has expected schema version;
- field has expected old value;
- dependency package/version is present;
- target has not already been patched by another layer;
- effective authority class matches expectation.

Conceptual example:

```toml
[[content_patches]]
op = "test"
kind = "material"
target = "freven.vanilla:materials/block/stone"
path = "render_layer"
expected = "solid"
```

If a precondition fails, the dependent patch must fail with a diagnostic instead
of silently applying to an unexpected entry.

## Deterministic order

Content composition should follow the same high-level selected stack as asset
resolution:

1. Product/install boundary selects allowed package roots.
2. Experience baseline contributes base content.
3. Experience stack layers apply in explicit stack order.
4. Required mods/content packs apply in dependency-resolved deterministic order.
5. Patch operations apply in explicit per-layer order.
6. Client-local cosmetic patches apply only where policy allows.
7. Effective content graph validates.
8. Compatibility fingerprints and generated caches are derived.

The resolver must not depend on filesystem traversal order, archive entry order,
hash-map iteration order, or accidental mod discovery order.

When two patches target the same field or keyed member, the schema/policy must
define one of:

- deterministic ordering;
- explicit override permission;
- idempotent merge behavior;
- conflict.

## Namespace and protected ownership

Packages own their namespaces.

Rules:

- a package can add and patch entries in its own namespace according to schema;
- patching another namespace requires explicit dependency and policy;
- patching `freven.vanilla:*` requires selected stack/package policy;
- protected namespaces may reject client-local or untrusted patches;
- a content pack cannot silently take ownership of another package's keys;
- diagnostics must name both the owner namespace and patching package.

This supports Vanilla mods and total conversions without making namespace theft
accidental.

## Authority and compatibility

Patch semantics must distinguish authority classes.

| Class | Meaning |
| --- | --- |
| Authoritative content | Affects gameplay, world/server compatibility, recipes, providers, block/item identity, gameplay state, or required visual bindings |
| Server-required visual/content binding | Presentation/content binding required by server/experience policy |
| Client-local cosmetic content | Presentation-only change allowed by policy |
| Generated cache | Derived output, never an authored patch target |
| Save/world state | Runtime persistence, migrated separately |

Rules:

- authoritative content changes may require compatibility fingerprints;
- authoritative content changes may require save/world migrations;
- client-local cosmetic patches must not alter gameplay identity or server-owned
  compatibility;
- generated cache is invalidated/rebuilt, not patched;
- save/world state is migrated, not patched like content source.

## Conflict diagnostics

Invalid composition should fail with actionable diagnostics.

Examples:

| Problem | Diagnostic should mention |
| --- | --- |
| Duplicate add | content kind, key, declaring package/layer, existing owner |
| Missing patch target | operation, kind, target key, patch file/layer |
| Wrong content kind | expected kind, actual kind, key |
| Forbidden namespace patch | target namespace, owner package, patching package, required policy |
| Field path missing | target key, path, schema rule |
| Type mismatch | target key, path, expected type, actual value |
| Duplicate append member | collection path, member key, existing owner |
| Ambiguous patch order | competing patches, target path, missing ordering rule |
| Forbidden authoritative change | authority class, server/experience policy, patching package |
| Missing dependency | required package/key/version |
| Save compatibility risk | changed authoritative key and missing migration/compatibility policy |
| Renderer/internal id used | field path and expected author-facing key |

Diagnostics should say whether the fix belongs in content data, dependency
metadata, stack order, namespace policy, schema, asset override policy, or a
save/world migration.

## Relationship to layered asset overrides

[LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md) decides which
whole visual asset declaration/file wins for an asset key.

This document decides how structured content entries are added, replaced,
patched, appended, disabled, hidden, and validated.

Examples:

- replacing a texture file for `freven.vanilla:textures/block/grass` belongs to
  asset override rules;
- changing a block visual field from one material key to another belongs to
  content patch semantics;
- appending one variant to a content family belongs to content patch semantics;
- rebuilding an atlas after either change belongs to generated cache handling.

## Relationship to creator-friendly schemas

This document defines semantics, not beginner syntax.

[CREATOR_CONTENT_SCHEMA_v1.md](CREATOR_CONTENT_SCHEMA_v1.md) defines the
creator-friendly schema direction, and
[DATA_DRIVEN_AUTHORING_LAYER_v1.md](DATA_DRIVEN_AUTHORING_LAYER_v1.md)
defines the practical authoring workflow for simpler files such as block, item,
recipe, material, model, or family definitions. Those files should compile down
to the same semantic operations described here.

That keeps beginner authoring simple without creating a second hidden content
composition model.

## DevKit guidance

DevKit should eventually be able to explain:

- the selected content layer stack;
- every content entry candidate and patch operation;
- the owner package/layer for each effective entry;
- why an add/replace/patch/append/disable operation succeeded or failed;
- which field path changed;
- which dependency or namespace policy allowed the change;
- whether the change is authoritative, server-required, cosmetic, or generated;
- which compatibility fingerprints changed;
- whether a save/world migration is required.

Diagnostics should point authors to authored content files, not generated cache,
runtime state, or renderer internals.

## Non-goals

This document does not define:

- final beginner-friendly content file syntax beyond the v1 schema direction;
- final block/item/recipe/material/model schemas;
- final visual asset override winner rules beyond the existing v1 document;
- final save/world migration format;
- marketplace policy or trust UI;
- runtime hot-patching of live worlds;
- renderer-internal ids, slots, or atlas coordinates.

Those are separate rc10 follow-up documents and implementation issues.
