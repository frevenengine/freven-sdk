use std::{collections::BTreeMap, sync::Arc};

use freven_block_sdk_types::BlockRuntimeId;
use freven_volumetric_sdk_types::{ColumnCoord, SectionY, WorldCellPos};

/// Contract for worldgen providers registered through SDK.
pub trait WorldGenProvider: Send + Sync {
    fn generate(
        &mut self,
        _request: &WorldGenRequest,
        _output: &mut WorldGenOutput,
    ) -> Result<(), WorldGenError> {
        Ok(())
    }
}

/// Worldgen provider factory init parameters.
///
/// Volumetric topology and addressing come from
/// `freven_volumetric_sdk_types`.
///
/// Standard block/profile ids are imported from
/// `freven_block_sdk_types`. This crate consumes that vocabulary for
/// worldgen convenience, but does not own it.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct WorldGenInit {
    pub seed: u64,
    pub world_id: Option<String>,
    /// Stable string-key -> standard block/profile runtime id mapping.
    ///
    /// The ids are block-layer vocabulary imported from
    /// `freven_block_sdk_types`.
    pub block_ids: BTreeMap<String, BlockRuntimeId>,
}

impl WorldGenInit {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            world_id: None,
            block_ids: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn block_id_by_key(&self, key: &str) -> Option<BlockRuntimeId> {
        self.block_ids.get(key).copied()
    }
}

/// Worldgen provider factory. One provider instance can be created per world/session.
pub type WorldGenFactory = Arc<dyn Fn(WorldGenInit) -> Box<dyn WorldGenProvider> + Send + Sync>;

/// Minimal worldgen request contract for one requested column.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WorldGenRequest {
    pub seed: u64,
    pub column: ColumnCoord,
}

impl WorldGenRequest {
    #[must_use]
    pub const fn new(seed: u64, column: ColumnCoord) -> Self {
        Self { seed, column }
    }

    #[must_use]
    pub const fn cx(&self) -> i32 {
        self.column.cx
    }

    #[must_use]
    pub const fn cz(&self) -> i32 {
        self.column.cz
    }
}

/// Terrain writes emitted by a worldgen provider.
///
/// Volumetric addressing is owned by `freven_volumetric_sdk_types`.
/// Standard block/profile ids are imported from `freven_block_sdk_types`.
/// `freven_world_api` does not own either lower-layer vocabulary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldTerrainWrite {
    FillSection {
        sy: SectionY,
        block_id: BlockRuntimeId,
    },
    FillBox {
        min: WorldCellPos,
        max: WorldCellPos,
        block_id: BlockRuntimeId,
    },
    SetBlock {
        pos: WorldCellPos,
        block_id: BlockRuntimeId,
    },
}

/// World-owned terrain generation output for one requested column.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WorldGenOutput {
    pub writes: Vec<WorldTerrainWrite>,
}

/// Worldgen contract error placeholder.
#[derive(Debug, thiserror::Error)]
#[error("worldgen error: {message}")]
pub struct WorldGenError {
    pub message: String,
}
