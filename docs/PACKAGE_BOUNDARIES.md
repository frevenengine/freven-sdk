# Package Boundaries and State Ownership

This document defines where Freven package metadata, active config, authored
content data, assets, generated cache, and save/world state belong.

It builds on [ARCHITECTURE.md](ARCHITECTURE.md). That document defines the
platform vocabulary. This document defines the file/state ownership model that
authors, SDK docs, DevKit validation, and future rc10 visual/data work should use.

## Core rule

Do not mix these categories:

- manifests describe package identity and declared capabilities;
- config schemas describe supported settings and defaults;
- active config selects values for one experience/stack/run;
- content data describes authored gameplay and visual definitions;
- assets provide referenced resource files;
- generated cache stores derived host/tooling output;
- save/world state stores runtime persistence.

A file can reference another category, but it should not become that category.
For example, `mod.toml` may reference `config.schema.toml`, but it is not active
runtime config. A material definition may reference a texture asset, but the
texture file is not the material definition. A generated atlas may be built from
textures, but it is not authored source.

## Source-of-truth table

| Category | Typical location | Owns | Must not own |
| --- | --- | --- | --- |
| Manifest | `mod.toml`, `experience.toml`, package/product manifests | package identity, version, dependencies, artifact kind, execution/trust policy, active sides/surfaces, entrypoint, capability requests, schema references, package metadata | active runtime values, authored content definitions, binary assets, generated cache, save/world state |
| Config schema | `config.schema.toml` next to the package manifest | supported setting keys, types, defaults, validation constraints, scopes, reload policy, authority | selected server/user values for a concrete run, save data, content definitions |
| Active config | selected experience or stack config tables | chosen values for one resolved experience/stack, per-mod effective runtime config | package identity, schema definition, save/world state, authored content source |
| Content data | `content/` | authored definitions such as blocks, items, recipes, providers, material definitions, model definitions, visual bindings, families, tags, gameplay data | binary resource bytes, user config, generated atlases/load plans, runtime save data |
| Assets | `assets/` | referenced resource files such as `png`, `ktx2`, `glb`, `gltf`, `wgsl`, sounds, fonts, localization source, and other resource data | package identity, active config, save/world state, generated cache as source of truth |
| Generated cache | host/devkit-owned cache directory | derived load plans, atlases, texture arrays, compiled/transcoded assets, fingerprints, validation indexes, incremental build outputs | authored source, user-visible configuration, durable world state |
| Save/world state | world/session save directory | runtime-persistent state, world data, player/session state, authoritative persisted gameplay state, migrations already applied to a save | shipped content defaults, package manifests, config schemas, generated cache |

## Manifests

A manifest is package metadata and declaration. For a mod package, `mod.toml`
declares identity, version, dependency requirements, artifact/execution/trust
metadata, side/surface hosting, entrypoint, capability requests, and references
to schema files.

Manifest example:

```toml
schema = 3
id = "example.hello"
version = "0.1.0"
artifact = "wasm_module"
execution = "wasm_guest"
trust = "sandboxed"
policy = "safe_guest"
surfaces = "server"
entry = "mod.wasm"
config_schema = "config.schema.toml"
```

Manifest rules:

- manifests are resolved before runtime start;
- manifests are not active runtime config;
- manifests may reference config schemas, content roots, asset roots, and
  executable artifacts;
- manifests should use safe relative paths for package-owned files;
- manifests should not be edited by runtime systems to persist play state;
- manifests should not define block/item/material/recipe data directly unless a
  future schema explicitly says a manifest field is package metadata for that
  purpose;
- dependency and capability decisions should be explicit and diagnosable.

`mod.toml [config]` is not a supported active runtime config path. Active values
belong to the selected experience or stack and are resolved before guest start.

## Config schema and active config

A config schema declares what can be configured. Active config chooses values.

`config.schema.toml` owns:

- setting keys;
- setting types;
- defaults;
- validation constraints;
- scope;
- reload policy;
- authority.

The selected experience or stack owns active values:

```toml
[config."example.hello"]
enabled = true
difficulty = "hard"
```

Runtime guests do not read manifest files, schema files, experience files, or
stack files directly. The host resolves the effective per-mod config and delivers
it through the guest start input.

See [MOD_CONFIG_v1.md](MOD_CONFIG_v1.md) for the detailed config schema,
override, validation, and guest delivery model.

## Content data

Content data is authored source for gameplay and visual definitions.

Examples:

```text
content/
  blocks/
  items/
  recipes/
  tags/
  providers/
  materials/
  models/
  visuals/
  families/
```

Content data can include definitions for blocks, items, recipes, providers,
material declarations, model declarations, block visual bindings, content
families, tags, and future data-driven gameplay definitions.

Content data rules:

- content data is authored source;
- content data should use namespaced identities;
- content data can reference assets by stable keys or package-relative paths
  according to the relevant schema;
- authoritative content data participates in compatibility identity;
- changes to authoritative content data may require save migrations or explicit
  compatibility decisions;
- content data should not store generated atlases, load plans, or runtime world
  state;
- content data should not rely on renderer-internal slots or host object ids.

The core visual asset identity, category, dependency, validation, and resolution
model is defined in [VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md).
Detailed content authoring schemas, override behavior, and patch/merge behavior
are separate rc10 documents. This document only defines where those files belong
and how they relate to state ownership.

## Assets

Assets are resource files referenced by content data, manifests, or tooling.

Examples:

```text
assets/
  textures/
  models/
  shaders/
  sounds/
  fonts/
  lang/
```

Common asset examples include `png`, `ktx2`, `glb`, `gltf`, `wgsl`, sound files,
fonts, and localization source.

Asset rules:

- assets are authored resource files;
- assets should be referenced through stable author-facing keys or safe
  package-relative paths;
- assets may be authoritative or cosmetic depending on the asset type, package
  role, server policy, and future override rules;
- renderer-internal slots, atlas coordinates, texture-array indexes, Bevy handles,
  and cache paths are not author-facing asset identities;
- an asset file is not the same thing as a material/model/content definition that
  references it.

A texture used by a gameplay-relevant block definition may be server-required.
A local cosmetic override may be client-local where policy allows. DevKit and
runtime diagnostics must distinguish those cases instead of treating every asset
change as equivalent.

## Generated cache

Generated cache is derived output owned by the host, engine, or DevKit.

Examples:

```text
.freven-cache/
  load-plans/
  atlases/
  texture-arrays/
  fingerprints/
  compiled-assets/
  validation-indexes/
```

Generated cache rules:

- cache may be deleted and rebuilt;
- cache should not be the only source of authored truth;
- cache may include fingerprints and derived compatibility data;
- cache paths are not public author-facing ids;
- cache can be packaged as an optimization only when tooling explicitly marks it
  as derived and safe to rebuild;
- stale cache must never override authoritative source validation.

Generated atlases, texture arrays, and load plans are host/runtime details. They
are useful for performance and diagnostics, but authors should target stable
content and asset keys.

## Save/world state

Save/world state is runtime-persistent data created by play or simulation.

Examples:

```text
worlds/
  <world_id>/
    world-state/
    chunks/
    players/
    sessions/
    migrations/
```

Save/world state rules:

- save state is not shipped content defaults;
- runtime systems write save state, not `content/` source files;
- save state may be bound to an experience id, mod set, content fingerprint, or
  compatibility manifest;
- authoritative content changes may require migrations;
- generated cache is not save state;
- save migrations should transform save/world state, not silently rewrite shipped
  package source.

## Compatibility classes

Freven tooling should classify package data into compatibility classes.

| Class | Examples | Compatibility impact |
| --- | --- | --- |
| Authoritative content | gameplay block definitions, item definitions, recipes, provider declarations, server-required material/model bindings | participates in server/world compatibility and may require migration |
| Server-required assets | assets required to render or validate authoritative content according to server policy | may participate in server-required fingerprints |
| Client-local cosmetic assets | local texture/theme/resource overrides allowed by policy | should not alter authoritative gameplay identity |
| Active config | selected values resolved for a run or stack | may affect startup, world restart, reconnect, or runtime behavior depending on schema reload policy |
| Generated cache | atlases, load plans, compiled/transcoded resources, validation indexes | rebuildable; should not be durable compatibility truth by itself |
| Save/world state | chunks, persisted gameplay state, player/session/world data | durable runtime state; migration target when authoritative definitions change |

A client-only texture override is not the same as changing a server-side block
definition. Changing a recipe is not the same as changing a shader. Rebuilding an
atlas is not a save migration.

## Override and patch ownership

This document defines ownership, not the full override algorithm.

High-level rules:

- manifests are selected and resolved; they are not patched accidentally by
  unrelated content packs;
- config schemas belong to the package that declares them;
- active config is authored by the selected experience/stack or a future explicit
  config layer;
- content data may be added, patched, disabled, or replaced only through explicit
  content patch semantics;
- assets may be overridden only through explicit asset layer rules;
- save/world state is migrated, not overridden like a content file;
- generated cache is invalidated/rebuilt, not patched as source.

Detailed layered asset override rules are defined in
[LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md).
Detailed content add/patch/replace/disable semantics are defined in
[CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md).

## Package layout examples

These examples show expected ownership boundaries. They are conceptual layouts;
installers and product packaging may embed the package root under instance,
experience, or product directories.

### Runtime-loaded Wasm mod

```text
mods/example.hello/
  mod.toml
  config.schema.toml
  mod.wasm
  content/
    blocks/
    recipes/
  assets/
    textures/
    sounds/
```

- `mod.toml` declares identity, dependency, execution, trust, surfaces,
  entrypoint, capabilities, and schema path.
- `config.schema.toml` declares supported settings and defaults.
- `content/` contains authored definitions.
- `assets/` contains referenced resources.
- Runtime world data does not get written back into this package.

### Content pack

```text
content-packs/example.visuals/
  mod.toml
  content/
    materials/
    models/
    visuals/
  assets/
    textures/
```

A content pack contains no executable guest artifact. It contributes content data
and assets through explicit content/asset rules.

A content pack can be useful for visual refreshes, resource packs, data-only
block/item additions, and standalone game content. It must not gain runtime
authority just because it provides files.

### Experience-owned bundled mod

```text
experiences/example.game/
  experience.toml
  mods/example.game.core/
    mod.toml
    config.schema.toml
    mod.wasm
    content/
    assets/
```

The experience owns the bundled package subtree and references the manifest by a
safe relative path. The package boundaries remain the same even though the mod is
bundled with the experience.

### Zero-Vanilla standalone product

```text
example-product/
  product.toml
  experiences/example.game/
    experience.toml
    content/
    assets/
    mods/example.game.core/
      mod.toml
      mod.wasm
      content/
      assets/
```

A standalone product may ship without Vanilla. It still uses the same manifest,
config, content, asset, cache, and save-state boundaries.

## DevKit validation guidance

DevKit and host diagnostics should point authors to the right boundary:

- unknown package id, dependency, trust, entrypoint, or surface problems:
  manifest diagnostic;
- unsupported setting key, invalid type, invalid enum, out-of-range value, or
  reload/authority mismatch: config schema or active config diagnostic;
- unknown block/item/recipe/material/model key: content data diagnostic;
- missing image/model/shader/sound/font file: asset diagnostic;
- stale atlas/load plan/fingerprint: generated cache invalidation diagnostic;
- incompatible existing world: save/world migration or compatibility diagnostic;
- renderer slot, atlas coordinate, or internal handle mentioned in authored data:
  author-facing API boundary diagnostic.

The diagnostic should explain which file category owns the fix instead of telling
authors to edit the wrong file.

## Non-goals

This document does not define:

- detailed texture/material/model/shader schemas beyond the shared visual asset
  model;
- the asset override algorithm beyond the v1 policy model;
- the content patch/merge algorithm beyond the v1 semantic model;
- final content-pack manifest schema;
- final generated cache directory names;
- final save migration format;
- package marketplace rules.

Those are separate rc10 follow-up documents and implementation issues.
