# Modular Content Manifests v1

This document defines the Freven rc10 contract for splitting large authored
content packs across multiple manifest files while preserving one deterministic
resolved content graph.

It builds on:

- [PACKAGE_BOUNDARIES.md](PACKAGE_BOUNDARIES.md): manifest, config, content,
  assets, generated cache, and save/world state ownership;
- [DATA_DRIVEN_AUTHORING_LAYER_v1.md](DATA_DRIVEN_AUTHORING_LAYER_v1.md):
  friendly authoring files that compile into the same semantic content model;
- [CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md): semantic add,
  replace, patch, append, disable, compatibility, and diagnostics model;
- [LAYERED_ASSET_OVERRIDES_v1.md](LAYERED_ASSET_OVERRIDES_v1.md): selected
  stack layering and override policy;
- [VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md): stable visual asset
  identity, dependency graph, and renderer-backend boundary;
- [CONTENT_VARIANT_FAMILY_SCHEMA_v1.md](CONTENT_VARIANT_FAMILY_SCHEMA_v1.md):
  deterministic source-time family expansion.

## Core rule

Modular manifests are an authoring-source convenience.

They do not create a second content model. The loader expands an explicit include
graph into one resolved content source for the selected content layer, then the
normal content validation, patch/merge, family expansion, asset dependency
validation, and runtime load-plan steps run exactly as if the entries had been
written in one file.

## Goals

- Make large content packs readable without one giant `content.manifest`.
- Keep source ordering deterministic and explicit.
- Preserve stable namespaced content and asset keys.
- Preserve layered override and patch/merge semantics.
- Give diagnostics exact source file and field paths.
- Keep generated cache, runtime ids, renderer slots, atlas coordinates, and save
  state out of author-facing files.
- Let Vanilla, mods, content packs, and standalone products use the same model.

## Non-goals

This document does not define:

- a scriptable build system;
- arbitrary imports from outside the content root;
- hidden filesystem traversal or glob order;
- runtime hot-patching of active worlds;
- renderer/runtime ids in source content;
- final engine or DevKit implementation details.

## Recommended package shape

A large package may use an index-style root manifest:

~~~text
content/
  content.manifest
  textures/
    terrain.toml
    glass.toml
  materials/
    terrain.toml
    glass.toml
  models/
    topsoil.toml
    framed_glass.toml
  visuals/
    terrain.toml
    glass.toml
  families/
    rock.toml
    soil_grass.toml
    colored_glass.toml
  tags/
    terrain.toml
~~~

The root manifest owns the package content graph entry point:

~~~toml
schema = 1

includes = [
  "textures/terrain.toml",
  "materials/terrain.toml",
  "models/topsoil.toml",
  "families/rock.toml",
  "families/soil_grass.toml",
  "tags/terrain.toml",
]
~~~

Small packages may still use a single `content.manifest`.

## Include path rules

Include paths are authored source paths, not runtime ids.

Rules:

- includes are an explicit ordered list;
- include paths are UTF-8 safe relative paths;
- absolute paths are invalid;
- `..` traversal is invalid;
- paths must stay inside the package content root;
- paths are resolved relative to the file that declares the include;
- symlink or canonicalization escape from the content root is invalid;
- include paths must name files, not directories;
- missing include files are hard validation errors;
- filesystem traversal order is never semantic ordering.

Globs are not part of v1. A future tool may generate an explicit include list,
but the checked-in semantic source must remain explicit and deterministic.

## Include graph rules

The include graph is explicit and acyclic.

Rules:

- a file may include other files;
- cycles are rejected with a diagnostic that prints the include stack;
- repeated inclusion of the same file in one layer is rejected unless a future
  schema explicitly defines idempotent includes;
- implementations should enforce a conservative include-depth limit and report it
  as a source authoring error;
- expansion order is depth-first in listed order;
- local entries in a file are considered before that file's child includes for
  provenance ordering;
- semantic add/replace/patch/append/disable behavior is still owned by
  [CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md), not by physical file
  order.

Physical file order may help diagnostics and stable fingerprints. It must not be
used as an implicit override mechanism.

## Allowed file contents

Included files contain the same semantic content tables as the root manifest.

Examples:

~~~toml
[[textures]]
key = "freven.vanilla:textures/soil_medium"
path = "textures/soil_medium.png"
sha256 = "..."

[[materials]]
key = "freven.vanilla:block/soil_medium"
texture = "freven.vanilla:textures/soil_medium"
fallback_debug_tint_rgba = 1867398655
~~~

or:

~~~toml
[[families]]
key = "freven.vanilla:families/soil_grass"

[families.axes.fertility]
values = ["poor", "medium", "rich"]

[families.axes.coverage]
values = ["bare", "sparse", "normal"]
~~~

A content kind is interpreted exactly the same whether it appears in the root
manifest or an included file.

## Layering and patch/merge semantics

A selected experience or stack still resolves content layers first.

For each layer:

1. Load the root content manifest.
2. Expand its include graph.
3. Validate all source entries with file/field provenance.
4. Convert source entries into semantic content operations.
5. Apply patch/merge rules for that layer.
6. Apply layered override policy across the selected stack.
7. Run family expansion.
8. Build validation indexes, asset graphs, load plans, and runtime tables.

Duplicate `add` entries in the same effective layer remain errors unless the
schema uses an explicit patch/replace/append operation. File order does not make
duplicate keys safe.

## Diagnostics

Diagnostics must identify both semantic and physical source context.

A good diagnostic should include:

- selected experience or stack id;
- content layer id and precedence;
- root manifest path;
- included file path;
- include stack when relevant;
- table/kind, such as `material`, `texture`, `family`, `visual`, or `tag`;
- semantic key, if known;
- field path, if known;
- expanded package-local shorthand, if relevant;
- suggested fix.

Example shape:

~~~text
content manifest include error
experience: freven.vanilla.visual_validation
layer: experience:base:0
root: content/content.manifest
include stack:
  content/content.manifest
  content/families/soil_grass.toml
problem: include path '../shared/materials.toml' escapes content root
field: includes[1]
fix: move the file under content/ and reference it with a safe relative path
~~~

## Fingerprints and cache

The resolved content fingerprint should include enough source data to detect
meaningful drift:

- root manifest path;
- include graph;
- included file paths;
- file contents or content hashes;
- semantic entries after include expansion;
- relevant patch/merge/family source data.

Fingerprints must not include:

- filesystem traversal order;
- modification time as source truth;
- generated cache paths;
- renderer slots;
- atlas coordinates;
- GPU handles.

Generated cache may store expanded manifests and dependency indexes, but cache is
derived output and can be deleted/rebuilt.

## Standalone packaging

Standalone product generation must package every source file that participates in
the content include graph, plus referenced asset bytes according to the package
boundary policy.

Packaging must not depend on scanning arbitrary directories. It should use the
validated include graph and asset dependency graph.

Missing included files or referenced assets are packaging errors.

## Compatibility notes

Changing include boundaries without changing semantic content should not by
itself change gameplay compatibility. However, implementations may include source
paths in developer-facing fingerprints for diagnostics and cache invalidation.

Changing semantic entries, family source data, material definitions, model
definitions, block tags, or authoritative gameplay content follows the normal
compatibility rules from the content patch/merge and selected-stack documents.

## Authoring guidance

Use one root manifest as the explicit index.

Recommended split for large visual packs:

~~~text
content/
  content.manifest
  textures/terrain.toml
  materials/terrain.toml
  models/topsoil.toml
  visuals/terrain.toml
  families/soil_grass.toml
  tags/terrain.toml
~~~

Prefer grouping by authoring concern, not by renderer/runtime implementation.

Good groups:

- textures
- materials
- models
- block visuals
- content families
- tags
- recipes/items/entities when those schemas exist

Bad groups:

- atlas slots
- internal material ids
- runtime block ids
- generated cache outputs

## Validation checklist

A complete implementation should validate:

- include paths are safe;
- include graph is acyclic;
- expansion is deterministic;
- missing include files fail before runtime start;
- duplicate semantic keys are diagnosed with all source paths;
- patch/merge operations retain exact source provenance;
- family expansion reports included file paths;
- DevKit check/inspect can explain included source entries;
- standalone packaging includes every needed source file and asset;
- runtime load plans do not expose include internals as author-facing ids.

## Relationship to existing Freven documents

This document defines physical source composition.

It does not replace:

- package/state ownership from [PACKAGE_BOUNDARIES.md](PACKAGE_BOUNDARIES.md);
- semantic content operations from [CONTENT_PATCH_MERGE_v1.md](CONTENT_PATCH_MERGE_v1.md);
- visual identity from [VISUAL_ASSET_MODEL_v1.md](VISUAL_ASSET_MODEL_v1.md);
- family expansion from [CONTENT_VARIANT_FAMILY_SCHEMA_v1.md](CONTENT_VARIANT_FAMILY_SCHEMA_v1.md);
- friendly source compilation from [DATA_DRIVEN_AUTHORING_LAYER_v1.md](DATA_DRIVEN_AUTHORING_LAYER_v1.md).

If there is disagreement, semantic content operations own meaning; modular
manifests only own explicit deterministic source-file composition.
