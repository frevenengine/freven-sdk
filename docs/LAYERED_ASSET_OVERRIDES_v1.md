# Layered Asset Overrides v1

This document defines deterministic layered override rules for Freven visual
assets.

It builds on:

- [ARCHITECTURE.md](ARCHITECTURE.md): engine / SDK / experience / mod /
  content-pack / standalone-product ownership;
- [PACKAGE_BOUNDARIES.md](PACKAGE_BOUNDARIES.md): manifest, config, content,
  assets, generated cache, and save/world state ownership;
- [VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md): visual asset identity,
  categories, dependency graph, validation, and renderer-backend boundaries.

The goal is to make asset replacement explicit, deterministic, diagnosable, and
safe for Vanilla mods, content packs, total conversions, servers, and zero-Vanilla
standalone games.

## Core rule

Asset overrides are not accidental filesystem behavior.

A package may always add assets in its own namespace. Replacing another package's
effective asset key requires an explicit override layer, explicit policy, and
diagnostics that can explain the winner and every shadowed candidate.

Generated cache is never an override source. It is rebuilt from the resolved
authored graph.

## Terminology

| Term | Meaning |
| --- | --- |
| Asset key | Stable author-facing `namespace:path` visual asset identity |
| Declared asset | Asset/content declaration provided by one package layer |
| Effective asset | The winning declaration after layering and override policy |
| Layer | Explicit source of package/content/asset declarations in a selected stack |
| Shadowed asset | A valid lower-priority declaration replaced by a higher-priority explicit override |
| Conflict | Two or more declarations cannot be resolved deterministically under policy |
| Cosmetic override | Client-local visual replacement allowed by policy that does not change authoritative gameplay identity |
| Server-required asset | Visual asset or binding required by server/experience policy |
| Generated cache | Rebuildable resolver/backend output derived from effective assets |

## Layer classes

The resolver should treat source layers as explicit classes.

| Layer class | Purpose | Can override |
| --- | --- | --- |
| Experience baseline | The selected experience's authored visual source | Its own namespace and explicitly declared dependencies according to policy |
| Experience stack layer | A deliberate authored layer over the base experience | Assets allowed by stack policy |
| Required mod/content pack | Server/experience-selected package | Own namespace; other namespaces only through explicit override declarations |
| Total conversion layer | Experience or stack that deliberately replaces the visual baseline | Namespaces declared in its policy |
| Client-local cosmetic pack | User/client-selected visual-only pack | Cosmetic asset classes allowed by server/experience policy |
| Generated cache | Derived output | Nothing; cache is invalidated/rebuilt |

Layer class is more important than incidental install path. A package inside an
experience tree, an instance mods directory, or a product bundle still needs an
explicit role in the selected stack.

## Deterministic order

A selected product/experience should resolve visual assets in this conceptual
order:

1. Product/install boundary selects allowed package roots.
2. Experience baseline provides the initial visual source.
3. Experience stack layers apply in explicit stack order.
4. Required mods/content packs apply in dependency-resolved deterministic order.
5. Explicit override declarations are evaluated.
6. Client-local cosmetic packs apply in explicit local order if policy allows.
7. The effective visual asset graph is validated.
8. Generated cache/load plans are built from the effective graph.

The resolver must not depend on:

- filesystem traversal order;
- archive entry order;
- platform-specific path sorting;
- hash-map iteration order;
- implicit Git checkout order;
- accidental mod discovery order.

If order matters, it must be authored or derived from declared dependencies and
stable tie-break rules.

## Override operations

Layered asset override v1 recognizes these high-level operations.

| Operation | Meaning |
| --- | --- |
| Add | Provide a new key not already declared by an earlier effective layer |
| Replace | Deliberately replace an existing key with the same asset category |
| Shadow | Effective result of a valid higher-priority replace |
| Disable | Remove a lower asset only if a future schema explicitly allows disable tombstones |
| Conflict | Fail resolution because policy cannot pick one effective asset safely |

Patch/merge of structured content data is not defined here. That is defined in
[CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md).

## Same-namespace and cross-namespace rules

Packages own their namespace.

Same-namespace rule:

- a package can add new keys in its own namespace;
- multiple declarations for the same key inside one package are duplicate-key
  errors unless a future schema has an explicit local override construct;
- a newer version of the same package may replace its own keys as part of normal
  package evolution.

Cross-namespace rule:

- replacing another namespace requires explicit override declaration;
- replacing `freven.vanilla:*` is allowed only through declared layer policy;
- replacing server-required assets may require server approval or exact resolved
  graph fingerprints;
- client-local cosmetic packs may not replace authoritative visual bindings;
- conflicts must report both owner packages and layer locations.

This keeps total conversions possible while preventing accidental namespace theft.

## Conceptual override declaration

The exact manifest/content syntax is a follow-up implementation detail. The
semantic model should support declarations shaped like this:

```toml
[[asset_overrides]]
target = "freven.vanilla:textures/block/grass"
replacement = "example.green:textures/block/grass_lush"
kind = "replace"
scope = "cosmetic"
reason = "resource-pack style grass refresh"
```

The important properties are:

- target key;
- replacement key or replacement declaration;
- operation kind;
- scope/authority class;
- owner package/layer;
- reason or diagnostic context.

A future schema may encode this in a manifest, content file, or stack layer. The
semantic requirement is that the override is explicit and inspectable.

## Authority classes

Override policy depends on authority class.

| Class | Override policy |
| --- | --- |
| Authoritative visual binding | Can be changed only by selected experience/stack/server-required package policy |
| Server-required visual asset | Client must provide the required effective key/hash or accepted equivalent |
| Client-local cosmetic asset | Client may replace only if policy allows and gameplay identity is unchanged |
| Generated visual cache | Never manually overridden; rebuilt from effective graph |

Examples:

- changing a block's visual binding from one model key to another may be
  authoritative if the server/experience requires that presentation;
- changing a local texture for the same accepted material key may be cosmetic if
  server policy allows it;
- changing a shader/effect to reveal hidden gameplay information is not a safe
  cosmetic override;
- replacing a generated atlas file is cache corruption, not an authored override.

## Server/client model

Servers and selected experiences may define visual asset policy.

A server may require:

- exact effective asset graph fingerprint;
- exact server-required texture/material/model/effect hashes;
- a named compatibility set;
- no client-local cosmetic packs;
- only cosmetic packs that target whitelisted classes;
- rejection of shaders/effects outside an allowed set.

A client may use local cosmetic packs only when the selected server/experience
policy allows it.

Client-local cosmetic overrides must not:

- alter authoritative content ids;
- change collision, selection, gameplay state, provider identity, or save data;
- replace server-required visual bindings unless allowed;
- bypass diagnostics by editing generated cache;
- expose renderer-internal slots or backend handles.

## Fingerprints and hashes

Fingerprints should describe resolved authored inputs, not generated cache paths.

Possible fingerprint scopes:

| Scope | Contents |
| --- | --- |
| Asset file hash | Bytes of one source asset file |
| Asset declaration hash | One material/model/effect/visual declaration |
| Package asset graph fingerprint | Effective graph for one package namespace |
| Server-required visual graph fingerprint | Effective graph subset required by server policy |
| Generated cache fingerprint | Derived output fingerprint used for invalidation only |

Rules:

- generated cache fingerprints are useful for invalidation, not durable
  compatibility truth by themselves;
- server-required fingerprints should be computed from effective authored inputs;
- diagnostics should show which key/file caused a mismatch;
- local cosmetic overrides should not alter authoritative graph fingerprints.

## Conflict diagnostics

The resolver should report clear conflicts.

Examples:

| Problem | Diagnostic should mention |
| --- | --- |
| Duplicate key in same layer | key, asset category, package, both files |
| Cross-namespace override without permission | target key, owner namespace, overriding package, required declaration/policy |
| Wrong asset type replacement | expected category, replacement category, target key |
| Forbidden client-local override | server/experience policy, pack id, key |
| Server-required hash mismatch | key, expected fingerprint, actual fingerprint, source package/file |
| Ambiguous layer order | involved packages/layers and missing ordering/dependency rule |
| Generated cache edited as source | cache path and source file/category that should be edited |
| Missing overridden target | override declaration and unresolved target key |

Diagnostics should explain whether the fix belongs in manifest/policy, content
data, asset files, stack order, or generated cache invalidation.

## Examples

### Vanilla texture cosmetic override

A client-local cosmetic pack wants greener grass:

```text
target:      freven.vanilla:textures/block/grass
replacement: example.green:textures/block/grass_lush
scope:       cosmetic
```

Allowed only if server/experience policy permits client-local texture overrides
for that asset class. It must not change gameplay identity.

### Vanilla material replacement by selected stack

A total conversion stack replaces a Vanilla material library:

```text
target:      freven.vanilla:materials/block/stone
replacement: example.total:materials/block/stone
scope:       selected-stack
```

Allowed only when the selected experience/stack policy explicitly declares that
the total conversion owns that replacement.

### Duplicate material conflict

Two required content packs both declare:

```text
example.shared:materials/block/marble
```

If neither package owns an explicit override relationship, resolution fails with a
duplicate/conflict diagnostic. The resolver must not choose based on filesystem
order.

### Generated cache is stale

A texture file changes and the atlas cache still contains the old bytes. The fix
is to invalidate/rebuild generated cache. Texture backend cache invalidation is
specified by [TEXTURE_BACKEND_PIPELINE_v1.md](TEXTURE_BACKEND_PIPELINE_v1.md). Authors should not edit the atlas file
as if it were source.

## Relationship to content patch/merge

Asset override rules decide which asset declaration/file wins for a visual asset
key.

They do not define deep merge behavior for structured gameplay/content data.

Examples that belong to content patch/merge, not this document:

- adding one face texture to an existing block visual object;
- changing only the `roughness` field inside a material declaration;
- appending one recipe ingredient;
- disabling one item variant inside a family;
- merging tag members.

This document can say whether one whole visual asset key replaces another.
[CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md) defines how structured
data changes within a key.

## Relationship to visual asset model

[VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md) defines identity,
categories, dependencies, validation, and renderer-backend boundaries.

This document adds deterministic layering and override policy on top of that
model. It does not change the meaning of visual asset keys or expose renderer
internals.

## DevKit guidance

DevKit should eventually be able to explain:

- the complete layer stack used for visual asset resolution;
- which package declared each candidate for a key;
- which candidate won and why;
- which candidates were shadowed;
- which override declaration allowed the replacement;
- whether the effective key is authoritative, server-required, cosmetic, or
  generated;
- which fingerprints are expected and actual;
- how to fix forbidden overrides, duplicate keys, stale cache, and server policy
  mismatches.

A good diagnostic points to the right source file instead of telling authors to
edit generated cache or renderer handles.

## Non-goals

This document does not define:

- material schema v1;
- block visual schema;
- texture atlas or texture-array packing;
- model format, defined by
  [MODEL_ASSET_FORMAT_v1.md](MODEL_ASSET_FORMAT_v1.md);
- shader/effect ABI;
- structured content patch/merge semantics defined by [CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md);
- save/world migration format;
- final marketplace/resource-pack policy UI.

Those are separate rc10 follow-up documents and implementation issues.
