//! Compile-time / builtin-facing block gameplay contracts.
//!
//! Ownership:
//! - block/profile vocabulary lives in `freven_block_sdk_types`
//! - runtime-loaded block mutation/query shapes live in `freven_block_guest`
//! - builtin/compile-time block gameplay traits and client block interaction
//!   surfaces live here

use freven_block_guest::BlockMutation;
use freven_block_sdk_types::BlockRuntimeId;

/// Read-only block-facing world view for gameplay handlers.
pub trait BlockWorldView {
    fn block(&self, wx: i32, wy: i32, wz: i32) -> Option<BlockRuntimeId>;
    fn is_solid(&self, block_id: BlockRuntimeId) -> bool;
}

/// Deterministic result of an authoritative block mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BlockMutationResult {
    Applied {
        old: BlockRuntimeId,
        new: BlockRuntimeId,
    },
    NotLoaded,
    OutOfBounds,
    Mismatch {
        current: BlockRuntimeId,
    },
    Rejected {
        message: String,
    },
}

/// Authoritative block mutation host surface.
pub trait BlockAuthority: BlockWorldView {
    fn try_apply(&mut self, mutation: &BlockMutation) -> BlockMutationResult;
}

/// Block face used by camera/hit and interaction APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ClientBlockFace {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

/// Camera ray in world space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientCameraRay {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

/// Camera cursor hit against a block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientCursorHit {
    pub block_pos: (i32, i32, i32),
    pub face: ClientBlockFace,
    pub distance_m: f32,
}

/// One predicted block edit hint (visual-only, not authoritative).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientPredictedEdit {
    pub pos: (i32, i32, i32),
    pub predicted_block_id: BlockRuntimeId,
}

impl ClientPredictedEdit {
    #[must_use]
    pub const fn clear_block(pos: (i32, i32, i32)) -> Self {
        Self {
            pos,
            predicted_block_id: BlockRuntimeId(0),
        }
    }
}

/// Authoritative block state correction for an action result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientActionEdit {
    pub pos: (i32, i32, i32),
    pub block_id: BlockRuntimeId,
}

/// Engine-provided camera and block-hit query surface.
pub trait ClientCameraHitProvider {
    fn camera_ray(&self) -> Option<ClientCameraRay>;

    fn authoritative_cursor_hit(&self, max_distance_m: f32) -> Option<ClientCursorHit>;

    fn predicted_cursor_hit(&self, max_distance_m: f32) -> Option<ClientCursorHit>;

    fn predicted_block_id_at(&self, pos: (i32, i32, i32)) -> Option<BlockRuntimeId>;

    fn authoritative_block_id_at(&self, pos: (i32, i32, i32)) -> Option<BlockRuntimeId>;
}
