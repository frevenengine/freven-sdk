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

/// Collision-facing standard block/profile semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockCollision {
    pub is_solid: bool,
}

/// Visibility-facing standard block/profile semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockVisibility {
    pub is_opaque: bool,
    pub render_layer: RenderLayer,
}

/// Client presentation metadata for a standard block/profile entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockMaterial {
    pub debug_tint_rgba: u32,
    pub material_id: u32,
}

/// Canonical standard block/profile descriptor.
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
