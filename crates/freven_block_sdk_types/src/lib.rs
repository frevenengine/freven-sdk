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

/// Client presentation metadata for a standard block/profile entry.
///
/// `debug_tint_rgba` is authored as `0xRRGGBBAA`.
///
/// `material_id` is the current low-level debug-palette slot. Normal mod
/// authors should not guess this value manually; use the `BlockDescriptor`
/// colored helpers, which set [`AUTO_DEBUG_MATERIAL_ID`] and let the host choose
/// a stable per-block palette slot.
///
/// Long term, real texture/material asset registration should live above this
/// legacy debug-palette field and resolve to renderer-internal slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockMaterial {
    pub debug_tint_rgba: u32,
    pub material_id: u32,
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
    }
}
