use freven_block_api::{BlockAuthority, BlockWorldView};
use freven_block_guest::{
    BlockQueryRequest, BlockQueryResponse, BlockServiceRequest, BlockServiceResponse,
};
use freven_block_sdk_types::BlockRuntimeId;
use freven_mod_api::{LogLevel, emit_log};

use crate::services::{Services, WorldServiceRequest, WorldServiceResponse};

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

/// Character-physics query service exposed to action handlers.
pub trait CharacterPhysicsQuery {
    fn player_position(&self, player_id: u64) -> Option<[f32; 3]>;
}

/// Stable action-dispatch context provided by runtime/server integration.
pub struct ActionContext<'a> {
    pub block_world: Option<&'a dyn BlockWorldView>,
    pub block_authority: Option<&'a mut dyn BlockAuthority>,
    pub character_physics: Option<&'a dyn CharacterPhysicsQuery>,
    pub services: Option<&'a mut dyn Services>,
    pub player_id: u64,
    pub at_input_seq: u32,
}

impl<'a> ActionContext<'a> {
    #[must_use]
    pub fn new(
        block_world: Option<&'a dyn BlockWorldView>,
        block_authority: Option<&'a mut dyn BlockAuthority>,
        character_physics: Option<&'a dyn CharacterPhysicsQuery>,
        services: Option<&'a mut dyn Services>,
        player_id: u64,
        at_input_seq: u32,
    ) -> Self {
        Self {
            block_world,
            block_authority,
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
    #[must_use]
    pub fn block_id_by_key(&mut self, key: &str) -> Option<BlockRuntimeId> {
        let services = self.services.as_deref_mut()?;
        match services.world_service(&WorldServiceRequest::Block(BlockServiceRequest::Query(
            BlockQueryRequest::BlockIdByKey {
                key: key.to_string(),
            },
        ))) {
            WorldServiceResponse::Block(BlockServiceResponse::Query(
                BlockQueryResponse::BlockIdByKey(value),
            )) => value,
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
