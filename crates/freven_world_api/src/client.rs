use std::sync::Arc;

use freven_block_sdk_types::BlockRuntimeId;

use crate::action::ActionKindId;

pub use freven_world_guest::ClientPlayerView as GuestClientPlayerView;

/// Mouse buttons for client input polling/consumption.
///
/// This enum is a convenience surface for common desktop bindings.
/// The primary cross-layer gameplay contract remains opaque input bytes
/// (`ClientControlOutput::input` / `CharacterControllerInput::input`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ClientMouseButton {
    Left,
    Right,
    Middle,
}

/// Keyboard keys for client input polling/consumption.
///
/// This enum is a convenience surface for common desktop bindings.
/// Prefer mod-defined opaque input payloads as the stable gameplay contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ClientKeyCode {
    KeyW,
    KeyA,
    KeyS,
    KeyD,
    KeyE,
    KeyQ,
    Space,
    Shift,
    Ctrl,
    Escape,
}

/// Block face used by camera/hit and interaction APIs.
///
/// This is convenience metadata for block-aligned interactions.
/// Action payload semantics are still owned by mod-defined opaque bytes.
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

/// One predicted world edit hint (visual-only, not authoritative).
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

/// Authoritative block state correction for an action result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientActionEdit {
    pub pos: (i32, i32, i32),
    pub block_id: BlockRuntimeId,
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

/// Lightweight player view for client-side presentation mods.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientPlayerView {
    pub player_id: u64,
    pub world_pos_m: (f32, f32, f32),
    pub is_local: bool,
}

/// Engine-provided client input surface.
pub trait ClientInputProvider {
    /// Raw device hold state for the current engine frame.
    fn mouse_button_down(&self, button: ClientMouseButton) -> bool;

    /// Raw render/update-frame press edge.
    ///
    /// This is frame-scoped engine input state, not a gameplay-tick latch. Gameplay systems that
    /// can tick at a different cadence must not rely on this for durable action submission.
    fn mouse_button_just_pressed(&self, button: ClientMouseButton) -> bool;

    /// Raw device hold state for the current engine frame.
    fn key_down(&self, key: ClientKeyCode) -> bool;

    /// Raw render/update-frame press edge.
    ///
    /// This is frame-scoped engine input state, not a gameplay-tick latch. Gameplay systems that
    /// can tick at a different cadence must not rely on this for durable action submission.
    fn key_just_pressed(&self, key: ClientKeyCode) -> bool;
    fn bind_mouse_button(&mut self, button: ClientMouseButton, owner: &str) -> bool;
    fn bind_key(&mut self, key: ClientKeyCode, owner: &str) -> bool;

    /// Consume one buffered mouse press for a gameplay-tick owner.
    ///
    /// Contract:
    /// - each successful consume corresponds to one raw press edge captured by the engine
    /// - presses are buffered across render frames until consumed or a gameplay reset clears them
    /// - multiple render-frame presses before the next gameplay tick must be delivered one-by-one
    /// - multiple gameplay ticks in one render frame must not re-consume the same raw edge
    fn consume_mouse_button_press(&mut self, button: ClientMouseButton, owner: &str) -> bool;

    /// Consume one buffered key press for a gameplay-tick owner.
    ///
    /// Semantics match `consume_mouse_button_press`.
    fn consume_key_press(&mut self, key: ClientKeyCode, owner: &str) -> bool;
}

/// Engine-provided raw device input state for client control providers.
pub trait ClientControlDeviceState {
    fn bind_mouse_button(&mut self, button: ClientMouseButton, owner: &str) -> bool;
    fn bind_key(&mut self, key: ClientKeyCode, owner: &str) -> bool;
    fn mouse_button_down(&self, button: ClientMouseButton, owner: &str) -> bool;
    fn key_down(&self, key: ClientKeyCode, owner: &str) -> bool;
    fn mouse_delta(&self) -> (f32, f32);
    fn cursor_locked(&self) -> bool;
    fn view_angles_deg(&self) -> (f32, f32);
}

/// Engine-provided camera and block-hit query surface.
pub trait ClientCameraHitProvider {
    fn camera_ray(&self) -> Option<ClientCameraRay>;

    /// Authoritative hit query against the last streamed base world state.
    ///
    /// Use this when gameplay needs to encode or validate a world target that must agree with
    /// server-side authoritative block state. This intentionally ignores pending local predicted
    /// edits so local overlay previews cannot retarget gameplay actions.
    fn authoritative_cursor_hit(&self, max_distance_m: f32) -> Option<ClientCursorHit>;

    /// Prediction-aware hit query against the client-visible world.
    ///
    /// This includes pending local predicted edits when they affect the current ray. Use it for
    /// previews, client-only UX, or debug tooling that intentionally wants overlay-aware picks.
    fn predicted_cursor_hit(&self, max_distance_m: f32) -> Option<ClientCursorHit>;

    /// Prediction-aware block query against the client-visible world.
    ///
    /// Use this for local preview/UI decisions, not as an authoritative submit gate.
    fn predicted_block_id_at(&self, pos: (i32, i32, i32)) -> Option<BlockRuntimeId>;

    /// Last authoritative streamed block state, excluding pending local predicted edits.
    ///
    /// This is still client-side knowledge and may be absent for unloaded terrain, but it is the
    /// correct query when gameplay needs the replicated base world rather than preview state.
    fn authoritative_block_id_at(&self, pos: (i32, i32, i32)) -> Option<BlockRuntimeId>;
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
