use freven_block_api::{ClientActionEdit, ClientPredictedEdit};
use std::sync::Arc;

use crate::action::ActionKindId;

/// Client-side action request submitted by gameplay/mods.
///
/// Giant-grade contract:
/// - mods describe intent (kind + payload) and optional predicted edits
/// - engine assigns `action_seq`, owns pending/retransmit/reconcile
#[derive(Debug, Clone)]
pub struct ClientActionRequest {
    /// Logical action kind id registered by the runtime/mods.
    pub action_kind_id: ActionKindId,

    /// Opaque action payload owned by the mod.
    ///
    /// `Arc<[u8]>` keeps clones cheap across layers.
    pub payload: Arc<[u8]>,

    /// Rule-A anchor: "apply before movement input seq N".
    pub at_input_seq: u32,

    /// Optional predicted edits for immediate visual feedback.
    pub predicted: Vec<ClientPredictedEdit>,
}

/// Local/engine-side rejection for `submit_action`.
///
/// This is NOT a server `ClientActionRejectReason`.
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum ClientActionSubmitError {
    #[error("cannot submit action: no active world stream")]
    NoActiveStream,

    #[error("cannot submit action: client is not in Play state")]
    NotInPlay,

    #[error("cannot submit action: too many pending actions")]
    TooManyPending,

    #[error("cannot submit action: payload too large (len={len}, limit={limit})")]
    PayloadTooLarge { len: usize, limit: usize },

    #[error("cannot submit action: {message}")]
    Other { message: String },
}

/// Action result reject reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ClientActionRejectReason {
    Unknown,
    NotLoaded,
    InvalidTarget,
    Duplicate,
    UnhandledActionKind,
}

/// Inbound action result event.
///
/// Notes:
/// - `(level_id, stream_epoch)` identify the stream the server associated this result with.
/// - Results MAY refer to a non-active stream (late reject / echo / stream mismatch).
///   Consumers are allowed to ignore non-active stream results.
#[derive(Debug, Clone)]
pub struct ClientActionResultEvent {
    /// Connection-scoped load session id (recipient-scoped).
    pub level_id: u32,

    /// Stream epoch for the action result.
    pub stream_epoch: u32,

    /// Rule-A anchor: "apply before movement input seq N".
    pub at_input_seq: u32,

    /// Monotonic per-connection action sequence (wrapping).
    pub action_seq: u32,

    /// True when the action was applied by the server.
    pub ok: bool,

    /// Reject reason when `ok == false`.
    pub reason: Option<ClientActionRejectReason>,

    /// Authoritative world edits produced by the server for this action.
    pub edits: Vec<ClientActionEdit>,
}

/// Engine-provided interaction request/result surface.
///
/// Giant-grade contract:
/// - mods submit intent (`ClientActionRequest`)
/// - engine assigns `action_seq`, owns pending/retransmit/reconcile
/// - mods observe outcomes via `poll_action_result` events
pub trait ClientInteractionProvider {
    fn active_stream(&self) -> Option<(u32, u32)>;

    fn next_input_seq(&self) -> u32;

    /// Submit an action request. Engine assigns and returns `action_seq`.
    ///
    /// Call sites must handle `Err(...)` explicitly. Local rejection is part of the normal engine
    /// contract boundary and should not be silently swallowed by gameplay code.
    fn submit_action(&mut self, req: ClientActionRequest) -> Result<u32, ClientActionSubmitError>;

    fn poll_action_result(&mut self) -> Option<ClientActionResultEvent>;
}
