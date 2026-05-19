//! Public standard block/profile vocabulary for Freven.
//!
//! Ownership:
//! - stable runtime ids for standard block/profile entries
//! - reusable collision / visibility / material descriptor vocabulary
//! - render-layer classification for standard block/profile presentation
//!
//! Non-responsibilities:
//! - volumetric topology, addressing, storage, or extraction
//! - generic world bootstrap / save / session truth
//! - registry / lookup ownership
//! - authority / prediction / manifest pipeline ownership
//! - vanilla-specific defaults, balance, or content policy
//!
//! This crate is the public owner of reusable standard block gameplay-facing
//! type surfaces that are not vanilla-specific.

use serde::{Deserialize, Serialize};

/// Stable runtime id for a standard block/profile entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockRuntimeId(pub u32);

/// Rendering layer classification for standard block/profile presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderLayer {
    Opaque,
    Cutout,
    Transparent,
}

/// Special author-facing material id for simple debug-colored blocks.
///
/// The current MVP renderer uses a 256-entry debug palette. Low-level callers may
/// still provide explicit palette slots, but normal mod authoring should prefer
/// the colored block helpers below. Those helpers use this sentinel so the host
/// can resolve the block to a stable per-runtime-block palette slot during
/// registry finalization / rendering.
///
/// This keeps raw renderer/palette slots out of the normal mod authoring path
/// while preserving the existing wire/schema shape.
pub const AUTO_DEBUG_MATERIAL_ID: u32 = u32::MAX;

/// Current debug palette width used by the MVP voxel renderer.
///
/// This is intentionally documented as the legacy/debug palette range, not the
/// long-term texture/material asset model.
pub const DEBUG_PALETTE_WIDTH: u32 = 256;

/// Maximum valid explicit debug palette material id for the current MVP renderer.
pub const MAX_EXPLICIT_DEBUG_MATERIAL_ID: u32 = DEBUG_PALETTE_WIDTH - 1;

/// Canonical block-local face names used by shape metadata.
///
/// This is gameplay/query vocabulary, not renderer mesh identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockShapeFace {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

impl BlockShapeFace {
    pub const ALL: [Self; 6] = [
        Self::Top,
        Self::Bottom,
        Self::North,
        Self::South,
        Self::East,
        Self::West,
    ];

    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
        }
    }
}

/// Axis-aligned block-local box in normalized voxel coordinates.
///
/// Bounds are authored in block-local `0.0..=1.0` coordinates. This type is the
/// reusable SDK vocabulary for simple collision, selection, and other block
/// shape query volumes. It intentionally does not describe render mesh triangles.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BlockShapeBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl BlockShapeBox {
    #[must_use]
    pub const fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    #[must_use]
    pub const fn full_block() -> Self {
        Self {
            min: [0.0, 0.0, 0.0],
            max: [1.0, 1.0, 1.0],
        }
    }

    #[must_use]
    pub const fn half_bottom() -> Self {
        Self {
            min: [0.0, 0.0, 0.0],
            max: [1.0, 0.5, 1.0],
        }
    }

    #[must_use]
    pub fn is_full_block(self) -> bool {
        self.min == [0.0, 0.0, 0.0] && self.max == [1.0, 1.0, 1.0]
    }

    #[must_use]
    pub fn is_valid(self) -> bool {
        self.min
            .into_iter()
            .chain(self.max)
            .all(|value| value.is_finite() && (0.0..=1.0).contains(&value))
            && self.min[0] < self.max[0]
            && self.min[1] < self.max[1]
            && self.min[2] < self.max[2]
    }
}

/// Per-side shape metadata for culling/support-style block semantics.
///
/// These flags are intentionally independent from material opacity and render
/// layer. A transparent/cutout block can still be physically side-solid, and an
/// opaque partial block must not automatically imply full-face occlusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockShapeSideMask {
    pub top: bool,
    pub bottom: bool,
    pub north: bool,
    pub south: bool,
    pub east: bool,
    pub west: bool,
}

impl BlockShapeSideMask {
    pub const EMPTY: Self = Self {
        top: false,
        bottom: false,
        north: false,
        south: false,
        east: false,
        west: false,
    };

    pub const FULL: Self = Self {
        top: true,
        bottom: true,
        north: true,
        south: true,
        east: true,
        west: true,
    };

    #[must_use]
    pub const fn get(self, face: BlockShapeFace) -> bool {
        match face {
            BlockShapeFace::Top => self.top,
            BlockShapeFace::Bottom => self.bottom,
            BlockShapeFace::North => self.north,
            BlockShapeFace::South => self.south,
            BlockShapeFace::East => self.east,
            BlockShapeFace::West => self.west,
        }
    }

    #[must_use]
    pub const fn with(mut self, face: BlockShapeFace, value: bool) -> Self {
        match face {
            BlockShapeFace::Top => self.top = value,
            BlockShapeFace::Bottom => self.bottom = value,
            BlockShapeFace::North => self.north = value,
            BlockShapeFace::South => self.south = value,
            BlockShapeFace::East => self.east = value,
            BlockShapeFace::West => self.west = value,
        }
        self
    }
}

/// Reusable canonical block shape semantics.
///
/// This is public SDK vocabulary shared by engine/runtime, game-owned authoring
/// profiles, and mod-facing APIs. It is not Vanilla-specific and it is not render
/// mesh identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockShapeDescriptor {
    pub occludes: BlockShapeSideMask,
    pub side_solid: BlockShapeSideMask,
    pub collision_boxes: Vec<BlockShapeBox>,
    pub selection_boxes: Vec<BlockShapeBox>,
}

impl BlockShapeDescriptor {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            occludes: BlockShapeSideMask::EMPTY,
            side_solid: BlockShapeSideMask::EMPTY,
            collision_boxes: Vec::new(),
            selection_boxes: Vec::new(),
        }
    }

    #[must_use]
    pub fn full_cube() -> Self {
        let full = BlockShapeBox::full_block();
        Self {
            occludes: BlockShapeSideMask::FULL,
            side_solid: BlockShapeSideMask::FULL,
            collision_boxes: vec![full],
            selection_boxes: vec![full],
        }
    }

    #[must_use]
    pub fn solid_non_occluding(boxes: impl Into<Vec<BlockShapeBox>>) -> Self {
        let boxes = boxes.into();
        Self {
            occludes: BlockShapeSideMask::EMPTY,
            side_solid: BlockShapeSideMask::FULL,
            collision_boxes: boxes.clone(),
            selection_boxes: boxes,
        }
    }

    #[must_use]
    pub fn with_occlusion(mut self, occludes: BlockShapeSideMask) -> Self {
        self.occludes = occludes;
        self
    }

    #[must_use]
    pub fn with_side_solid(mut self, side_solid: BlockShapeSideMask) -> Self {
        self.side_solid = side_solid;
        self
    }

    #[must_use]
    pub fn with_selection_boxes(mut self, selection_boxes: impl Into<Vec<BlockShapeBox>>) -> Self {
        self.selection_boxes = selection_boxes.into();
        self
    }

    #[must_use]
    pub fn occludes_face(&self, face: BlockShapeFace) -> bool {
        self.occludes.get(face)
    }

    pub fn validate(&self) -> Result<(), BlockShapeValidationError> {
        for (index, bounds) in self.collision_boxes.iter().copied().enumerate() {
            if !bounds.is_valid() {
                return Err(BlockShapeValidationError::InvalidCollisionBox { index, bounds });
            }
        }

        for (index, bounds) in self.selection_boxes.iter().copied().enumerate() {
            if !bounds.is_valid() {
                return Err(BlockShapeValidationError::InvalidSelectionBox { index, bounds });
            }
        }

        Ok(())
    }
}

/// Validation errors for SDK block shape descriptors.
#[derive(Debug, Clone, PartialEq)]
pub enum BlockShapeValidationError {
    InvalidCollisionBox { index: usize, bounds: BlockShapeBox },
    InvalidSelectionBox { index: usize, bounds: BlockShapeBox },
}

/// Collision-facing reusable standard block/profile semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockCollision {
    pub is_solid: bool,
}

/// Visibility-facing reusable standard block/profile semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockVisibility {
    pub is_opaque: bool,
    pub render_layer: RenderLayer,
}

/// Author-facing visual source for a standard block/profile entry.
///
/// `DebugColor` is the current simple/debug fallback path.
///
/// `MaterialKey` means the block was authored against a stable namespaced
/// material key. The host still resolves that key to renderer-internal
/// palette/atlas/texture-array slots during load; mod authors must not depend
/// on raw renderer ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BlockVisualKind {
    #[default]
    DebugColor,
    MaterialKey,
}

/// Empty material-key hash used by pure debug-colored blocks.
pub const NO_MATERIAL_KEY_HASH: u64 = 0;

/// FNV-1a offset basis for stable material-key hashing.
const MATERIAL_KEY_HASH_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

/// FNV-1a prime for stable material-key hashing.
const MATERIAL_KEY_HASH_PRIME: u64 = 0x0000_0100_0000_01B3;

/// Compute the stable compact hash used to carry namespaced resource-key identity.
///
/// This is shared by material keys, block tag keys, and other compact
/// `namespace:path` identities that need deterministic ABI/runtime fingerprints.
#[must_use]
pub const fn namespaced_key_hash(key: &str) -> u64 {
    let bytes = key.as_bytes();
    let mut hash = MATERIAL_KEY_HASH_OFFSET_BASIS;
    let mut i = 0usize;

    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(MATERIAL_KEY_HASH_PRIME);
        i += 1;
    }

    hash
}

/// Compute the stable compact hash used to carry namespaced material-key identity.
///
/// The original namespaced key remains the authoring/debug/error-reporting
/// surface. This hash is only a compact deterministic ABI/runtime identity and
/// must not be treated as a renderer slot.
#[must_use]
pub const fn material_key_hash(key: &str) -> u64 {
    namespaced_key_hash(key)
}

/// Compute the stable compact hash used to carry namespaced block-tag identity.
///
/// The readable tag key remains the public authoring/debug surface. This hash is
/// only a compact deterministic ABI/runtime identity and must not be treated as
/// a runtime block id, renderer slot, or gameplay-specific meaning.
#[must_use]
pub const fn block_tag_key_hash(key: &str) -> u64 {
    namespaced_key_hash(key)
}

/// Returns true when `key` is a stable Freven namespaced resource key.
///
/// The accepted MVP shape is `namespace:path`, where the namespace allows
/// lowercase ASCII letters, digits, `_`, `-`, and `.`, and the path also allows
/// `/` for folders.
#[must_use]
pub fn is_valid_namespaced_key(key: &str) -> bool {
    let Some((namespace, path)) = key.split_once(':') else {
        return false;
    };

    !namespace.is_empty()
        && !path.is_empty()
        && namespace.bytes().all(is_valid_namespace_byte)
        && path.bytes().all(is_valid_resource_path_byte)
}

/// Returns true when `key` is a stable Freven namespaced material key.
///
/// The accepted MVP shape is `namespace:path`, where the namespace allows
/// lowercase ASCII letters, digits, `_`, `-`, and `.`, and the path also allows
/// `/` for folders. This intentionally mirrors resource-key style authoring
/// instead of exposing renderer-local numeric ids.
#[must_use]
pub fn is_valid_material_key(key: &str) -> bool {
    is_valid_namespaced_key(key)
}

/// Returns true when `key` is a stable Freven namespaced block tag key.
///
/// Block tags are semantic content groupings such as `freven:stones` or
/// `modid:gas_permeable`. They are not block runtime ids, renderer ids, or
/// hardcoded engine gameplay concepts.
#[must_use]
pub fn is_valid_block_tag_key(key: &str) -> bool {
    is_valid_namespaced_key(key)
}

#[inline]
fn is_valid_namespace_byte(b: u8) -> bool {
    b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-' | b'.')
}

#[inline]
fn is_valid_resource_path_byte(b: u8) -> bool {
    is_valid_namespace_byte(b) || b == b'/'
}

/// Client presentation metadata for a standard block/profile entry.
///
/// `debug_tint_rgba` is authored as `0xRRGGBBAA` and remains the simple fallback
/// color for debug-colored blocks and material-key blocks whose real material
/// has not been resolved yet.
///
/// `material_id` is the current low-level debug-palette slot. Normal mod
/// authors should not guess this value manually; use the `BlockDescriptor`
/// colored/material helpers, which set [`AUTO_DEBUG_MATERIAL_ID`] and let the
/// host choose stable internal slots.
///
/// `material_key_hash` is a deterministic compact identity for a namespaced
/// material key. It is not a renderer slot; the host must resolve the authored
/// key through the material registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockMaterial {
    pub debug_tint_rgba: u32,
    pub material_id: u32,
    #[serde(default)]
    pub visual_kind: BlockVisualKind,
    #[serde(default)]
    pub material_key_hash: u64,
}

/// Canonical reusable standard block/profile descriptor.
///
/// This is not generic world truth and not neutral volumetric truth.
/// It belongs to the standard block/profile layer above volumetric foundations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockDescriptor {
    pub collision: BlockCollision,
    pub visibility: BlockVisibility,
    pub material: BlockMaterial,
}

impl BlockDescriptor {
    #[must_use]
    pub const fn new(
        is_solid: bool,
        is_opaque: bool,
        render_layer: RenderLayer,
        debug_tint_rgba: u32,
        material_id: u32,
    ) -> Self {
        Self {
            collision: BlockCollision { is_solid },
            visibility: BlockVisibility {
                is_opaque,
                render_layer,
            },
            material: BlockMaterial {
                debug_tint_rgba,
                material_id,
                visual_kind: BlockVisualKind::DebugColor,
                material_key_hash: NO_MATERIAL_KEY_HASH,
            },
        }
    }

    #[must_use]
    pub const fn air() -> Self {
        Self::new(false, false, RenderLayer::Opaque, 0, 0)
    }

    /// Define a simple debug-colored cube without manually choosing a palette slot.
    ///
    /// The host resolves [`AUTO_DEBUG_MATERIAL_ID`] to a stable per-block debug
    /// palette slot. This is the recommended current authoring path for simple
    /// visible custom blocks until real texture/material asset registration lands.
    #[must_use]
    pub const fn colored_cube(
        is_solid: bool,
        is_opaque: bool,
        render_layer: RenderLayer,
        debug_tint_rgba: u32,
    ) -> Self {
        Self::new(
            is_solid,
            is_opaque,
            render_layer,
            debug_tint_rgba,
            AUTO_DEBUG_MATERIAL_ID,
        )
    }

    /// Define a normal opaque solid debug-colored cube.
    #[must_use]
    pub const fn solid_colored_cube(debug_tint_rgba: u32) -> Self {
        Self::colored_cube(true, true, RenderLayer::Opaque, debug_tint_rgba)
    }

    /// Define a block that references a namespaced material key.
    ///
    /// The key is validated and hashed for compact ABI/runtime identity. The
    /// host resolves the original key through Material Registry v1; until that
    /// registry exists, `fallback_debug_tint_rgba` keeps the block visible in
    /// the current debug-palette renderer.
    ///
    /// # Panics
    ///
    /// Panics if `material_key` is not a valid `namespace:path` key.
    #[must_use]
    pub fn material_cube(
        is_solid: bool,
        is_opaque: bool,
        render_layer: RenderLayer,
        material_key: &str,
        fallback_debug_tint_rgba: u32,
    ) -> Self {
        assert!(
            is_valid_material_key(material_key),
            "invalid Freven material key; expected namespace:path"
        );
        Self::material_cube_hashed(
            is_solid,
            is_opaque,
            render_layer,
            material_key_hash(material_key),
            fallback_debug_tint_rgba,
        )
    }

    /// Define a block that references an already-hashed material key.
    ///
    /// This is primarily useful for generated code and tests. Normal authoring
    /// should use [`Self::material_cube`] or [`Self::solid_material_cube`] with
    /// the readable namespaced key.
    #[must_use]
    pub const fn material_cube_hashed(
        is_solid: bool,
        is_opaque: bool,
        render_layer: RenderLayer,
        material_key_hash: u64,
        fallback_debug_tint_rgba: u32,
    ) -> Self {
        Self {
            collision: BlockCollision { is_solid },
            visibility: BlockVisibility {
                is_opaque,
                render_layer,
            },
            material: BlockMaterial {
                debug_tint_rgba: fallback_debug_tint_rgba,
                material_id: AUTO_DEBUG_MATERIAL_ID,
                visual_kind: BlockVisualKind::MaterialKey,
                material_key_hash,
            },
        }
    }

    /// Define a normal opaque solid cube backed by a namespaced material key.
    #[must_use]
    pub fn solid_material_cube(material_key: &str, fallback_debug_tint_rgba: u32) -> Self {
        Self::material_cube(
            true,
            true,
            RenderLayer::Opaque,
            material_key,
            fallback_debug_tint_rgba,
        )
    }

    /// Define a non-solid transparent debug-colored cube.
    #[must_use]
    pub const fn non_solid_colored_cube(debug_tint_rgba: u32) -> Self {
        Self::colored_cube(false, false, RenderLayer::Transparent, debug_tint_rgba)
    }

    /// Define a solid cutout debug-colored cube.
    #[must_use]
    pub const fn cutout_colored_cube(debug_tint_rgba: u32) -> Self {
        Self::colored_cube(true, false, RenderLayer::Cutout, debug_tint_rgba)
    }

    /// Define a solid transparent debug-colored cube.
    #[must_use]
    pub const fn transparent_colored_cube(debug_tint_rgba: u32) -> Self {
        Self::colored_cube(true, false, RenderLayer::Transparent, debug_tint_rgba)
    }

    /// Override the material id with an explicit legacy debug-palette slot.
    ///
    /// Prefer the colored helpers for new mod authoring. This exists for
    /// builtin/legacy/manual palette experiments that intentionally need a fixed
    /// slot.
    #[must_use]
    pub const fn with_explicit_debug_material_id(mut self, material_id: u32) -> Self {
        self.material.material_id = material_id;
        self
    }

    #[must_use]
    pub const fn uses_auto_debug_material_id(self) -> bool {
        self.material_id() == AUTO_DEBUG_MATERIAL_ID
    }

    #[must_use]
    pub const fn is_solid(self) -> bool {
        self.collision.is_solid
    }

    #[must_use]
    pub const fn is_opaque(self) -> bool {
        self.visibility.is_opaque
    }

    #[must_use]
    pub const fn render_layer(self) -> RenderLayer {
        self.visibility.render_layer
    }

    #[must_use]
    pub const fn debug_tint_rgba(self) -> u32 {
        self.material.debug_tint_rgba
    }

    #[must_use]
    pub const fn material_id(self) -> u32 {
        self.material.material_id
    }

    #[must_use]
    pub const fn visual_kind(self) -> BlockVisualKind {
        self.material.visual_kind
    }

    #[must_use]
    pub const fn uses_material_key(self) -> bool {
        matches!(self.visual_kind(), BlockVisualKind::MaterialKey)
    }

    #[must_use]
    pub const fn material_key_hash(self) -> u64 {
        self.material.material_key_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_colored_cube_uses_auto_debug_material_slot() {
        let def = BlockDescriptor::solid_colored_cube(0x3C7A_52FF);

        assert!(def.is_solid());
        assert!(def.is_opaque());
        assert_eq!(def.render_layer(), RenderLayer::Opaque);
        assert_eq!(def.debug_tint_rgba(), 0x3C7A_52FF);
        assert_eq!(def.material_id(), AUTO_DEBUG_MATERIAL_ID);
        assert!(def.uses_auto_debug_material_id());
    }

    #[test]
    fn explicit_debug_material_id_is_still_available_for_low_level_callers() {
        let def =
            BlockDescriptor::solid_colored_cube(0x8080_80FF).with_explicit_debug_material_id(7);

        assert_eq!(def.material_id(), 7);
        assert!(!def.uses_auto_debug_material_id());
        assert_eq!(def.visual_kind(), BlockVisualKind::DebugColor);
        assert_eq!(def.material_key_hash(), NO_MATERIAL_KEY_HASH);
    }

    #[test]
    fn solid_material_cube_uses_namespaced_key_hash_and_debug_fallback() {
        let key = "freven.test:block/green";
        let def = BlockDescriptor::solid_material_cube(key, 0x2EA0_43FF);

        assert!(def.is_solid());
        assert!(def.is_opaque());
        assert_eq!(def.render_layer(), RenderLayer::Opaque);
        assert_eq!(def.debug_tint_rgba(), 0x2EA0_43FF);
        assert_eq!(def.material_id(), AUTO_DEBUG_MATERIAL_ID);
        assert_eq!(def.visual_kind(), BlockVisualKind::MaterialKey);
        assert!(def.uses_material_key());
        assert_eq!(def.material_key_hash(), material_key_hash(key));
    }

    #[test]
    fn block_shape_full_cube_has_full_occlusion_collision_and_selection() {
        let shape = BlockShapeDescriptor::full_cube();

        assert!(shape.occludes_face(BlockShapeFace::North));
        assert!(shape.occludes_face(BlockShapeFace::Top));
        assert_eq!(shape.collision_boxes, vec![BlockShapeBox::full_block()]);
        assert_eq!(shape.selection_boxes, vec![BlockShapeBox::full_block()]);
        shape.validate().expect("full cube shape should validate");
    }

    #[test]
    fn block_shape_half_block_can_be_solid_without_occluding_faces() {
        let half = BlockShapeBox::half_bottom();
        let shape = BlockShapeDescriptor::solid_non_occluding(vec![half]);

        assert!(!shape.occludes_face(BlockShapeFace::Top));
        assert!(shape.side_solid.get(BlockShapeFace::Top));
        assert_eq!(shape.collision_boxes, vec![half]);
        assert_eq!(shape.selection_boxes, vec![half]);
        shape.validate().expect("half block shape should validate");
    }

    #[test]
    fn block_shape_supports_multiple_collision_boxes() {
        let post = BlockShapeBox::new([0.375, 0.0, 0.375], [0.625, 1.0, 0.625]);
        let arm = BlockShapeBox::new([0.0, 0.375, 0.375], [1.0, 0.625, 0.625]);

        let shape = BlockShapeDescriptor::solid_non_occluding(vec![post, arm]);

        assert_eq!(shape.collision_boxes.len(), 2);
        assert_eq!(shape.selection_boxes.len(), 2);
        shape.validate().expect("multi-box shape should validate");
    }

    #[test]
    fn block_shape_validation_rejects_invalid_bounds() {
        let invalid = BlockShapeBox::new([0.0, 0.0, 0.0], [1.2, 1.0, 1.0]);
        let shape = BlockShapeDescriptor::solid_non_occluding(vec![invalid]);

        assert!(matches!(
            shape.validate(),
            Err(BlockShapeValidationError::InvalidCollisionBox { index: 0, .. })
        ));
    }

    #[test]
    fn material_key_validation_requires_namespace_and_path() {
        assert!(is_valid_material_key("freven.test:block/green"));
        assert!(is_valid_material_key("example-mod:block_01"));

        assert!(!is_valid_material_key("missing_namespace"));
        assert!(!is_valid_material_key(":missing_namespace"));
        assert!(!is_valid_material_key("missing_path:"));
        assert!(!is_valid_material_key("Upper:block"));
        assert!(!is_valid_material_key("example:block space"));
    }

    #[test]
    fn material_key_hash_is_stable() {
        assert_eq!(
            material_key_hash("freven.test:block/green"),
            material_key_hash("freven.test:block/green")
        );
        assert_ne!(
            material_key_hash("freven.test:block/green"),
            material_key_hash("freven.test:block/brown")
        );
    }
    #[test]
    fn block_tag_key_validation_uses_namespaced_resource_shape() {
        assert!(is_valid_block_tag_key("freven:stones"));
        assert!(is_valid_block_tag_key("freven:terrain/solids"));
        assert!(is_valid_block_tag_key("example.mod:gas_permeable"));
        assert!(!is_valid_block_tag_key("missing_namespace"));
        assert!(!is_valid_block_tag_key(":missing_namespace"));
        assert!(!is_valid_block_tag_key("freven:"));
        assert!(!is_valid_block_tag_key("Freven:stones"));
        assert!(!is_valid_block_tag_key("freven:bad tag"));
    }

    #[test]
    fn block_tag_key_hash_is_stable_and_shared_with_namespaced_hash() {
        let key = "freven:stones";
        assert_eq!(block_tag_key_hash(key), namespaced_key_hash(key));
        assert_eq!(block_tag_key_hash(key), block_tag_key_hash(key));
        assert_ne!(block_tag_key_hash(key), block_tag_key_hash("freven:soils"));
    }
}
