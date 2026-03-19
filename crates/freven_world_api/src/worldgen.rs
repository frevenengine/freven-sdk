use std::{collections::BTreeMap, sync::Arc};

use freven_block_sdk_types::BlockRuntimeId;

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
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct WorldGenInit {
    pub seed: u64,
    pub world_id: Option<String>,
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

/// Minimal worldgen request contract placeholder.
#[derive(Debug, Default, Clone)]
pub struct WorldGenRequest {
    pub seed: u64,
    pub cx: i32,
    pub cz: i32,
}

/// World-owned terrain writes emitted by a worldgen provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldTerrainWrite {
    FillSection {
        sy: i8,
        block_id: BlockRuntimeId,
    },
    FillBox {
        min: (i32, i32, i32),
        max: (i32, i32, i32),
        block_id: BlockRuntimeId,
    },
    SetBlock {
        pos: (i32, i32, i32),
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
