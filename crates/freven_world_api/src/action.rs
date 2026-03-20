use freven_block_sdk_types::BlockRuntimeId;
use freven_mod_api::{LogLevel, emit_log};

use crate::services::{
    Services, WorldMutation, WorldQueryRequest, WorldQueryResponse, WorldServiceRequest,
    WorldServiceResponse,
};

/// Stable id for a logical player action kind.
///
/// This id is runtime/mod-facing and independent of transport packet variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionKindId(pub u16);

impl ActionKindId {
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }
}

/// Read-only view of an inbound player action command.
#[derive(Debug, Clone, Copy)]
pub struct ActionCmdView<'a> {
    pub action_kind: ActionKindId,
    pub level_id: u32,
    pub stream_epoch: u32,
    pub seq: u32,
    pub at_input_seq: u32,
    pub payload: &'a [u8],
}

/// Read-only world state visible to authoritative action handlers.
///
/// This trait consumes standard block/profile vocabulary from
/// `freven_block_sdk_types`; it does not own that vocabulary.
pub trait WorldView {
    fn block(&self, wx: i32, wy: i32, wz: i32) -> Option<BlockRuntimeId>;
    fn is_solid(&self, block_id: BlockRuntimeId) -> bool;
}

/// Deterministic result of an authoritative world mutation requested by an action handler.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum WorldMutationResult {
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

/// Server-authoritative world mutation service exposed to action handlers.
///
/// `WorldMutation` may currently reference block-facing shapes here, but
/// block/profile type ownership remains in `freven_block_sdk_types`.
pub trait WorldAuthority: WorldView {
    fn try_apply(&mut self, mutation: &WorldMutation) -> WorldMutationResult;
}

/// Character-physics query service exposed to action handlers.
pub trait CharacterPhysicsQuery {
    fn player_position(&self, player_id: u64) -> Option<[f32; 3]>;
}

/// Stable action-dispatch context provided by runtime/server integration.
pub struct ActionContext<'a> {
    pub world: Option<&'a dyn WorldView>,
    pub authority: Option<&'a mut dyn WorldAuthority>,
    pub character_physics: Option<&'a dyn CharacterPhysicsQuery>,
    pub services: Option<&'a mut dyn Services>,
    pub player_id: u64,
    pub at_input_seq: u32,
}

impl<'a> ActionContext<'a> {
    #[must_use]
    pub fn new(
        world: Option<&'a dyn WorldView>,
        authority: Option<&'a mut dyn WorldAuthority>,
        character_physics: Option<&'a dyn CharacterPhysicsQuery>,
        services: Option<&'a mut dyn Services>,
        player_id: u64,
        at_input_seq: u32,
    ) -> Self {
        Self {
            world,
            authority,
            character_physics,
            services,
            player_id,
            at_input_seq,
        }
    }

    pub fn log(&mut self, level: LogLevel, message: impl AsRef<str>) {
        let _ = &self.services;
        emit_log(level, message);
    }

    /// Resolve a registered standard block/profile id by stable string key.
    ///
    /// The returned id is block-layer vocabulary imported from
    /// `freven_block_sdk_types`; `freven_world_api` only exposes a
    /// world-facing consumer/service surface for it.
    #[must_use]
    pub fn block_id_by_key(&mut self, key: &str) -> Option<BlockRuntimeId> {
        let services = self.services.as_deref_mut()?;
        match services.world_service(&WorldServiceRequest::Query(
            WorldQueryRequest::BlockIdByKey {
                key: key.to_string(),
            },
        )) {
            WorldServiceResponse::Query(WorldQueryResponse::BlockIdByKey(value)) => value,
            _ => None,
        }
    }
}

/// Server-side action handler contract for runtime/mod dispatch.
pub trait ActionHandler: Send + Sync {
    fn handle(&mut self, ctx: &mut ActionContext<'_>, cmd: &ActionCmdView<'_>) -> ActionOutcome;
}

/// Minimal action handler result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionOutcome {
    Applied,
    Rejected,
}
