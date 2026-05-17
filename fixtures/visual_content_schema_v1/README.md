# Visual Content Schema Conformance Fixtures v1

These fixtures are the canonical SDK-owned contract examples for the rc10 visual content stack.

They are intentionally outside renderer, DevKit, Boot, and Vanilla source trees so every repo can consume the same examples without inventing its own shape.

## Ownership

- freven-sdk owns the public semantic vocabulary, fixture layout, namespaced examples, and docs/tests that keep the fixtures stable.
- freven-engine consumes valid fixtures as load-plan/model/material/tint/light contract tests and invalid fixtures as diagnostics targets.
- freven-devkit uses fixtures for friendly content/assets check diagnostics, templates, and inspector examples.
- freven-boot uses fixtures to verify selected-stack visual content reaches the client startup/install boundary.
- freven-vanilla uses fixtures as the baseline shape for authored visual bindings and generated family content.

## Phase rule

Family expansion happens before runtime meshing.

Intended pipeline:

    authored family/source content
    -> deterministic generated block/material/model/visual entries
    -> resolved visual asset graph
    -> engine load plan
    -> client visual mesh table install
    -> meshing/rendering

Renderer slots, atlas coordinates, texture-array layers, GPU handles, Bevy/wgpu handles, runtime block ids, and generated cache paths are not valid authored fixture fields.

## Valid fixtures

- valid/cube_all: simple cube model and block visual using one all material slot.
- valid/cube_faces: per-face cube visual with top, side, and bottom material slots.
- valid/cuboid_parts_framed_glass: custom cuboid-parts model with frame and pane slots.
- valid/grass_tint_slots: slot/face-level tint metadata for grass-like visuals.
- valid/emissive_material_no_light: emissive material appearance without block light emission.
- valid/emissive_material_with_light: emissive material plus explicit light emission metadata.
- valid/families: rock, soil/grass, and colored-glass family expansion examples.

## Invalid fixtures

- invalid/missing_model: visual references an unresolved model key.
- invalid/unknown_material_slot: visual binds a material slot not declared by its model.
- invalid/invalid_tint_source: visual declares an unsupported tint source.
- invalid/invalid_family_combination: family has contradictory allow/skip combination policy.
- invalid/renderer_internal_leak: authored content contains renderer/backend/internal fields.

## Contract stability

These files are examples, not generated cache. They should remain small, namespaced, readable, and deterministic.

Follow-up repos should reference these fixtures in tests or docs instead of copying ad-hoc visual examples.
