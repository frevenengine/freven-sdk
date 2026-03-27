//! Volumetric-owned public contracts for deterministic world generation.
//!
//! Responsibilities:
//! - define the provider trait used by builtin mods and runtime hosts
//! - keep volumetric topology/addressing truth imported from
//!   `freven_volumetric_sdk_types`
//! - consume standard block/profile vocabulary from `freven_block_sdk_types`
//!   without claiming ownership of that gameplay layer
//! - remain independent from generic world/experience registration so that
//!   volumetric worldgen can be embedded by multiple world stacks

use std::{collections::BTreeMap, sync::Arc};

use freven_block_sdk_types::BlockRuntimeId;
use freven_volumetric_sdk_types::{ColumnCoord, SectionY, WorldCellPos};
use serde::{Deserialize, Serialize};

/// Contract for volumetric worldgen providers registered through SDK.
///
/// Canonical concurrency mode is `serial_session`: one provider instance is
/// session-owned and `generate` must not be invoked concurrently on that same
/// provider/session.
///
/// `Send + Sync` are memory-safety / host-integration bounds only. They do not
/// authorize shared-instance parallel worldgen execution, and they must not be
/// read as permission for overlapping `generate` calls on one provider.
///
/// See `docs/WORLDGEN_PROVIDER_CONCURRENCY_v1.md` for the canonical contract.
pub trait WorldGenProvider: Send + Sync {
    /// Generate terrain writes for one requested column.
    ///
    /// Current canonical execution is `serial_session`, so a host must not
    /// overlap `generate` calls on the same provider/session. Any future
    /// widening must use an explicit isolated-job contract rather than shared
    /// concurrent access to one provider instance.
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
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(default)]
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

/// Worldgen provider factory.
///
/// The factory object itself is `Send + Sync` so host registries can store and
/// share it safely. That does not widen provider execution semantics.
///
/// One provider instance can be created per world/session, and the canonical
/// execution contract for that provider remains `serial_session`. Future
/// widening, if ever activated, must use isolated-job semantics rather than
/// shared concurrent `generate` on one returned provider instance.
pub type WorldGenFactory = Arc<dyn Fn(WorldGenInit) -> Box<dyn WorldGenProvider> + Send + Sync>;

/// Minimal worldgen request contract for one requested column.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Advisory bootstrap-oriented metadata emitted by a worldgen provider.
///
/// This family is transport-independent semantic contract. Hosts may consume or
/// ignore it when bootstrapping a world's initial spawn state.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldGenBootstrapOutput {
    /// Advisory hint for selecting an initial world bootstrap spawn.
    pub initial_world_spawn_hint: Option<InitialWorldSpawnHint>,
}

/// Advisory initial world bootstrap spawn hint emitted by worldgen.
///
/// This hint is for initial world bootstrap semantics only. It is not a generic
/// respawn policy, runtime spawn service, or lifecycle-owned spawn contract.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct InitialWorldSpawnHint {
    /// Suggested world-space feet position in meters.
    ///
    /// This is the character feet position, not the collider center, capsule
    /// origin, or any other body-local reference point.
    pub feet_position: [f32; 3],
}

/// World-owned terrain generation output for one requested column.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldGenOutput {
    pub writes: Vec<WorldTerrainWrite>,
    /// Advisory bootstrap-oriented metadata for initial world bring-up.
    pub bootstrap: WorldGenBootstrapOutput,
}

/// Worldgen contract error placeholder.
#[derive(Debug, thiserror::Error)]
#[error("worldgen error: {message}")]
pub struct WorldGenError {
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::{
        InitialWorldSpawnHint, WorldGenBootstrapOutput, WorldGenOutput, WorldTerrainWrite,
    };
    use freven_block_sdk_types::BlockRuntimeId;

    #[test]
    fn worldgen_output_postcard_roundtrip_preserves_bootstrap_hint() {
        let output = WorldGenOutput {
            writes: vec![WorldTerrainWrite::FillSection {
                sy: 0.into(),
                block_id: BlockRuntimeId(7),
            }],
            bootstrap: WorldGenBootstrapOutput {
                initial_world_spawn_hint: Some(InitialWorldSpawnHint {
                    feet_position: [12.5, 65.0, -3.25],
                }),
            },
        };

        let encoded = postcard::to_allocvec(&output).expect("postcard encode");
        let decoded: WorldGenOutput = postcard::from_bytes(&encoded).expect("postcard decode");

        assert_eq!(decoded, output);
    }

    #[test]
    fn worldgen_output_default_has_no_bootstrap_hint() {
        let output = WorldGenOutput::default();

        assert!(output.bootstrap.initial_world_spawn_hint.is_none());
        assert!(output.writes.is_empty());
    }
}
