use serde::{Deserialize, Serialize};

/// Global runtime id for a block type/state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockRuntimeId(pub u32);

impl From<u8> for BlockRuntimeId {
    #[inline]
    fn from(v: u8) -> Self {
        Self(v as u32)
    }
}

/// Rendering layer classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderLayer {
    Opaque,
    Cutout,
    Transparent,
}

/// Minimal block definition needed by meshing and rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDef {
    pub is_solid: bool,
    pub is_opaque: bool,
    pub render_layer: RenderLayer,
    pub debug_tint_rgba: u32,
    pub material_id: u32,
}
