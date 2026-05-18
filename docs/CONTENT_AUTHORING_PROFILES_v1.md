# Content Authoring Profiles v1

This document defines the SDK-level boundary for game-owned content authoring
profiles.

Freven has one generic canonical content graph for engine/runtime consumption.
Games, modes, total conversions, and standalone products may define their own
creator-facing source schemas on top of that graph.

Vanilla may choose a blocktypes/worldproperties/shapes workflow. Another game may
choose a prototype pipeline, a resource-pack convention, or a custom domain
schema.

The engine must not hardcode Vanilla's authoring format.

## Core model

The intended pipeline is:

    Game/mode authoring source
      -> profile compiler / expander / validator
    Canonical Freven content graph
      -> engine/runtime registries, load plans, caches, and handles

The authoring profile is the friendly source layer. The canonical graph is the
stable backend boundary.

A profile can make authoring pleasant without creating a second runtime content
model.

## Why this exists

The canonical content graph is good for:

- deterministic loading;
- layer/override resolution;
- validation;
- source provenance;
- package fingerprints;
- generated cache;
- runtime load plans;
- stable diagnostics.

It is not always the best human-facing editing format.

A Vanilla block author should be able to think in terms of a blocktype, variant
groups, textures, shapes, drops, tags, sounds, and gameplay behavior hooks. They
should not need to understand engine registry tables, renderer slots, internal
material slots, atlas pages, or generated cache files.

## Ownership

| Layer | Owns |
| --- | --- |
| Engine/runtime | loading, resolving, validating canonical graph, renderer/backend registries, cache, runtime handles |
| SDK | public vocabulary for authoring profile contracts, canonical graph boundary, provenance, diagnostics expectations |
| Game/mode/profile | friendly source schema, conventions, defaults, compile rules, profile-specific validation |
| Boot/DevKit | profile compile/check/explain/update tooling |
| Mods/content packs | source files targeting the selected game/mode profile or low-level canonical graph by explicit choice |

## Canonical content graph

The canonical graph is the backend representation consumed by engine/runtime and
tooling.

Current rc10 graph areas include:

- texture declarations;
- material declarations;
- model declarations;
- block visual bindings;
- content families and deterministic generated entries;
- block tags;
- content patch/merge operations;
- layered content/asset overrides.

Future graph areas may include items, recipes, loot, entities, behaviors, UI,
effects, sounds, biome data, or other gameplay content.

The graph must remain:

- deterministic;
- namespaced;
- explicit after expansion;
- validated before runtime use;
- diagnosable back to authored source;
- free of renderer/runtime ids in author-authored files.

## Authoring profiles

An authoring profile is a game/mode-owned source schema plus compile/expand rules
that produce canonical graph declarations.

Examples:

- Vanilla blocktype profile:
  - blocktypes/
  - worldproperties/
  - shapes/
  - textures/
  - tags/
- Resource-pack-style profile:
  - conventional textures/models/sounds folders;
  - pack metadata;
  - override layers.
- Prototype-style profile:
  - data/prototype entrypoints;
  - deterministic staged expansion;
  - final canonical graph output.
- Custom standalone game profile:
  - product-specific entities, items, visuals, rules, or data tables.

Profiles are optional. A small package may target the canonical graph directly.

## Profile selection

An experience or stack should be able to select which profile owns a source tree.

Conceptually:

    [content]
    root = "content"
    manifest = "content.manifest"

    [content.authoring]
    profile = "freven.vanilla:blocktypes_v1"

The exact manifest/config shape is intentionally left to implementation issues.
The SDK boundary is:

- profile selection must be explicit;
- profile ids must be namespaced;
- profile versioning must be visible;
- profile output must compile to the canonical graph;
- diagnostics must report both authoring source and canonical output where useful.

## Source provenance

Profile compilers must preserve provenance.

Diagnostics should be able to answer:

- which profile was selected;
- which source file was read;
- which authoring kind was parsed;
- which field path failed;
- which canonical declaration was generated;
- which semantic key was produced;
- which included/generated source path was involved;
- what command can explain or fix the issue.

Example diagnostic shape:

    error: duplicate generated material key
    profile: freven.vanilla:blocktypes_v1
    source: content/blocktypes/grass.toml
    field: textures.side
    generated declaration: materials
    key: freven.vanilla:block/grass_side
    first source: content/blocktypes/grass.toml
    second source: content/blocktypes/soil.toml
    fix: rename the generated key, override intentionally, or move shared material ownership to a common source file

## Low-level canonical authoring remains valid

The canonical content manifest remains a valid advanced authoring format.

It is appropriate for:

- tests;
- generated output;
- low-level content packs;
- engine feature validation;
- advanced mods;
- bridge layers from custom tools;
- profiles that intentionally expose graph-level control.

But it should not be the only authoring UX that games can offer to modders.

## Vanilla profile direction

Vanilla should own its own friendly schema.

A future Vanilla profile may include source files such as:

    content/blocktypes/rock.toml
    content/blocktypes/soil.toml
    content/blocktypes/glass.toml
    content/worldproperties/rock.toml
    content/worldproperties/fertility.toml
    content/worldproperties/grass_coverage.toml
    content/shapes/block/cube.toml
    content/shapes/block/topsoil.toml
    content/tags/terrain.toml

Those files are Vanilla-owned authoring source. They compile into the same
canonical Freven graph consumed by the engine.

A zero-Vanilla standalone game must be able to ignore this profile and use its
own profile or the canonical graph directly.

## Mods and compatibility

A mod should declare what it targets:

- the engine/SDK canonical graph directly;
- a selected game-owned profile;
- both, when it intentionally mixes low-level and profile-level authoring.

A Vanilla mod targeting the Vanilla blocktype profile should not silently become
dependent on engine internals. A standalone game mod should not be forced to use
Vanilla profile files.

Compatibility should be checked at the profile boundary:

- profile id;
- profile version;
- generated canonical key stability;
- dependency namespaces;
- profile-specific feature gates;
- canonical graph compatibility.

## Generated output and cache

Profile compiler output may be materialized for inspection, packaging, caching,
or debugging.

Generated output must not become the human source of truth unless explicitly
declared by the profile.

Rules:

- authored profile source stays source;
- canonical generated graph can be rebuilt;
- generated cache is host/tooling output;
- runtime load plans are not authoring source;
- save/world state is not source;
- renderer handles, material slots, atlas coordinates, and runtime ids are not
  valid authoring fields.

## Relation to existing SDK docs

This document does not replace the existing data-driven authoring layer.

Instead:

- this document defines the profile boundary;
- DATA_DRIVEN_AUTHORING_LAYER_v1.md describes practical friendly source patterns;
- MODULAR_CONTENT_MANIFESTS_v1.md describes explicit include organization for
  canonical manifest source;
- CONTENT_VARIANT_FAMILY_SCHEMA_v1.md describes deterministic family expansion;
- CONTENT_PATCH_MERGE_v1.md describes semantic patch/merge rules;
- visual/material/model docs define canonical visual graph semantics.

## Non-goals

This document does not implement:

- Vanilla blocktype schema;
- Boot content compile command;
- DevKit templates;
- scripting language bindings;
- every future gameplay schema;
- runtime hot reload;
- save migration format.

Those belong in implementation issues.

## Acceptance checklist

A new authoring profile design should answer:

- Who owns the profile?
- How is the profile selected?
- What source files does it read?
- What canonical declarations does it produce?
- How are generated keys derived?
- How are overrides and layers handled?
- How is provenance preserved?
- How are errors explained?
- Can another game ignore this profile?
- Are engine/runtime ids kept out of authoring source?
