//! Stable SDK contracts for Freven experiences and compile-time mods.
//!
//! Responsibilities:
//! - define experience/mod descriptors used by boot/runtime layers
//! - expose deterministic registration surfaces (components/messages/worldgen/modnet)
//! - define stable hook contexts and registration errors
//! - act as the compile-time facade over the canonical declaration model exposed by `freven_guest`
//!
//! Extension guidance:
//! - add new registries behind stable string keys
//! - keep hook/context types engine-agnostic
//! - avoid leaking runtime/transport implementation details

use std::{cell::RefCell, ffi::c_void, sync::Arc, time::Duration};

use serde::de::DeserializeOwned;

pub use freven_guest::{
    CharacterControllerDeclaration, ClientControlProviderDeclaration,
    ClientNameplateDrawCmd as GuestClientNameplateDrawCmd,
    ClientPlayerView as GuestClientPlayerView, GuestCallbacks as ModCallbackModel,
    GuestRegistration as ModDeclarationModel, LifecycleHooks as LifecycleCallbackModel, LogLevel,
    LogPayload, MessageHooks as MessageCallbackModel, ModConfigDocument as GuestModConfigDocument,
    ModConfigFormat as GuestModConfigFormat, ProviderHooks as ProviderCallbackModel,
    RuntimeCharacterPhysicsRequest, RuntimeClientControlRequest, RuntimeCommandOutput,
    RuntimeEntityTarget, RuntimeLevelRef, RuntimeObservabilityRequest, RuntimeOutput,
    RuntimePresentationOutput, RuntimeReadRequest, RuntimeServiceRequest, RuntimeServiceResponse,
    RuntimeSessionInfo, RuntimeSessionSide, RuntimeSideRequest, WorldCommand, WorldGenDeclaration,
};
pub use freven_sdk_types::blocks::{BlockDef, BlockRuntimeId, RenderLayer};
pub use freven_sdk_types::{blocks, voxel};

/// Engine-owned replicated component keys.
pub mod engine_components {
    /// Optional per-player display name payload (UTF-8 bytes).
    pub const PLAYER_NAMEPLATE_TEXT: &str = "freven.engine:player_nameplate_text";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostExecutionKind {
    Builtin,
    Wasm,
    Native,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogCallbackFamily {
    StartClient,
    StartServer,
    TickClient,
    TickServer,
    ClientMessages,
    ServerMessages,
    Action,
    Worldgen,
    CharacterControllerInit,
    CharacterControllerStep,
    ClientControlSample,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostLogContext {
    pub mod_id: String,
    pub execution: HostExecutionKind,
    pub side: RuntimeSessionSide,
    pub runtime_session_id: u64,
    pub source: Option<String>,
    pub artifact: Option<String>,
    pub trust: Option<String>,
    pub policy: Option<String>,
    pub callback: Option<LogCallbackFamily>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostLogRecord {
    pub payload: LogPayload,
    pub context: HostLogContext,
}

pub type ObservabilityEmitFn =
    unsafe fn(ctx: *mut c_void, level: LogLevel, message_ptr: *const u8, message_len: usize);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ObservabilityBridge {
    pub ctx: *mut c_void,
    pub emit: Option<ObservabilityEmitFn>,
}

impl ObservabilityBridge {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            ctx: core::ptr::null_mut(),
            emit: None,
        }
    }
}

thread_local! {
    static OBSERVABILITY_BRIDGE: RefCell<ObservabilityBridge> =
        const { RefCell::new(ObservabilityBridge::empty()) };
}

pub fn emit_log(level: LogLevel, message: impl AsRef<str>) {
    let message = message.as_ref();
    OBSERVABILITY_BRIDGE.with(|slot| {
        let bridge = *slot.borrow();
        let Some(emit) = bridge.emit else {
            return;
        };
        unsafe {
            emit(bridge.ctx, level, message.as_ptr(), message.len());
        }
    });
}

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

/// Server world-read service exposed to action handlers.
pub trait ActionWorldRead {}

/// Deterministic result of a compare-and-set world edit requested by an action handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ActionWorldEditResult {
    Applied { old: u8, new: u8 },
    NotLoaded,
    OutOfBounds,
    Mismatch { current: u8 },
}

/// Server-authoritative world-edit service exposed to action handlers.
pub trait ActionWorldEdit {
    fn block_world(&self, wx: i32, wy: i32, wz: i32) -> u8;
    fn is_solid_block_id(&self, block_id: u8) -> bool;
    fn try_set_block_world_if(
        &mut self,
        wx: i32,
        wy: i32,
        wz: i32,
        expected_old: u8,
        new_id: u8,
    ) -> ActionWorldEditResult;
}

/// Character-physics query service exposed to action handlers.
pub trait CharacterPhysicsQuery {
    fn player_position(&self, player_id: u64) -> Option<[f32; 3]>;
}

/// Stable action-dispatch context provided by runtime/server integration.
pub struct ActionContext<'a> {
    pub world_read: Option<&'a dyn ActionWorldRead>,
    pub world_edit: Option<&'a mut dyn ActionWorldEdit>,
    pub character_physics: Option<&'a dyn CharacterPhysicsQuery>,
    pub services: Option<&'a mut dyn Services>,
    pub player_id: u64,
    pub at_input_seq: u32,
}

impl<'a> ActionContext<'a> {
    #[must_use]
    pub fn new(
        world_read: Option<&'a dyn ActionWorldRead>,
        world_edit: Option<&'a mut dyn ActionWorldEdit>,
        character_physics: Option<&'a dyn CharacterPhysicsQuery>,
        services: Option<&'a mut dyn Services>,
        player_id: u64,
        at_input_seq: u32,
    ) -> Self {
        Self {
            world_read,
            world_edit,
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

/// Execution side for a runtime instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Client,
    Server,
}

/// Side support declared by a compile-time mod.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModSide {
    Client,
    Server,
    Both,
}

impl ModSide {
    #[must_use]
    pub fn matches(self, side: Side) -> bool {
        matches!(
            (self, side),
            (Self::Both, _) | (Self::Client, Side::Client) | (Self::Server, Side::Server)
        )
    }
}

/// Compile-time convenience experience specification.
///
/// `config` is a top-level table keyed by mod id. Each mod receives its own value.
#[derive(Clone)]
pub struct ExperienceSpec {
    pub id: String,
    pub mods: Vec<ModDescriptor>,
    pub default_worldgen: Option<String>,
    pub default_character_controller: Option<String>,
    pub default_client_control_provider: Option<String>,
    pub config: toml::Table,
}

impl ExperienceSpec {
    #[must_use]
    pub fn mod_config(&self, mod_id: &str) -> Option<&toml::Value> {
        self.config.get(mod_id)
    }
}

/// Mod registration entrypoint type.
pub type ModRegisterFn = for<'a> fn(&'a mut ModContext<'a>);

/// Compile-time mod descriptor used by an experience.
#[derive(Clone)]
pub struct ModDescriptor {
    pub id: &'static str,
    pub version: &'static str,
    pub side: ModSide,
    pub register: ModRegisterFn,
}

/// Backend implemented by runtime for registration operations.
pub trait ModContextBackend {
    fn register_block(&mut self, key: &str, def: BlockDef)
    -> Result<BlockId, ModRegistrationError>;
    fn register_component(
        &mut self,
        key: &str,
        codec: ComponentCodec,
    ) -> Result<ComponentId, ModRegistrationError>;
    fn register_message(
        &mut self,
        key: &str,
        config: MessageConfig,
    ) -> Result<MessageId, ModRegistrationError>;
    fn register_worldgen(
        &mut self,
        key: &str,
        factory: WorldGenFactory,
    ) -> Result<WorldGenId, ModRegistrationError>;
    fn register_character_controller(
        &mut self,
        key: &str,
        factory: CharacterControllerFactory,
    ) -> Result<CharacterControllerId, ModRegistrationError>;
    fn register_client_control_provider(
        &mut self,
        key: &str,
        factory: ClientControlProviderFactory,
    ) -> Result<ClientControlProviderId, ModRegistrationError>;
    fn register_channel(
        &mut self,
        key: &str,
        config: ChannelConfig,
    ) -> Result<ChannelId, ModRegistrationError>;
    fn register_action_handler(
        &mut self,
        action_kind: ActionKindId,
        handler: Box<dyn ActionHandler>,
    ) -> Result<(), ModRegistrationError>;
    fn register_action_kind(&mut self, key: &str) -> Result<ActionKindId, ModRegistrationError>;
    fn on_start_client(&mut self, hook: StartClientHook);
    fn on_start_server(&mut self, hook: StartServerHook);
    fn on_tick_client(&mut self, hook: TickClientHook);
    fn on_tick_server(&mut self, hook: TickServerHook);
    fn on_client_messages(&mut self, hook: ClientMessagesHook);
    fn on_server_messages(&mut self, hook: ServerMessagesHook);
}

/// Stable SDK-facing registration context passed to mods.
pub struct ModContext<'a> {
    side: Side,
    mod_id: &'a str,
    experience_id: &'a str,
    config: &'a toml::Value,
    backend: &'a mut dyn ModContextBackend,
}

impl<'a> ModContext<'a> {
    #[must_use]
    pub fn new(
        side: Side,
        mod_id: &'a str,
        experience_id: &'a str,
        config: &'a toml::Value,
        backend: &'a mut dyn ModContextBackend,
    ) -> Self {
        Self {
            side,
            mod_id,
            experience_id,
            config,
            backend,
        }
    }

    #[must_use]
    pub fn side(&self) -> Side {
        self.side
    }

    #[must_use]
    pub fn mod_id(&self) -> &str {
        self.mod_id
    }

    #[must_use]
    pub fn experience_id(&self) -> &str {
        self.experience_id
    }

    #[must_use]
    pub fn config(&self) -> &toml::Value {
        self.config
    }

    pub fn config_typed<T: DeserializeOwned>(&self) -> Result<T, ModConfigError> {
        self.config
            .clone()
            .try_into()
            .map_err(|source| ModConfigError::Deserialize {
                mod_id: self.mod_id.to_string(),
                source,
            })
    }

    pub fn register_block(
        &mut self,
        key: &str,
        def: BlockDef,
    ) -> Result<BlockId, ModRegistrationError> {
        self.backend.register_block(key, def)
    }

    pub fn register_component(
        &mut self,
        key: &str,
        codec: ComponentCodec,
    ) -> Result<ComponentId, ModRegistrationError> {
        self.backend.register_component(key, codec)
    }

    pub fn register_message(&mut self, key: &str) -> Result<MessageId, ModRegistrationError> {
        self.backend.register_message(key, MessageConfig::default())
    }

    pub fn register_message_type(
        &mut self,
        key: &str,
        config: MessageConfig,
    ) -> Result<MessageId, ModRegistrationError> {
        self.backend.register_message(key, config)
    }

    pub fn register_worldgen(
        &mut self,
        key: &str,
        factory: impl Fn(WorldGenInit) -> Box<dyn WorldGenProvider> + Send + Sync + 'static,
    ) -> Result<WorldGenId, ModRegistrationError> {
        self.backend.register_worldgen(key, Arc::new(factory))
    }

    pub fn register_character_controller(
        &mut self,
        key: &str,
        factory: impl Fn(CharacterControllerInit) -> Box<dyn CharacterController>
        + Send
        + Sync
        + 'static,
    ) -> Result<CharacterControllerId, ModRegistrationError> {
        self.backend
            .register_character_controller(key, Arc::new(factory))
    }

    pub fn register_client_control_provider(
        &mut self,
        key: &str,
        factory: impl Fn(ClientControlProviderInit) -> Box<dyn ClientControlProvider>
        + Send
        + Sync
        + 'static,
    ) -> Result<ClientControlProviderId, ModRegistrationError> {
        self.backend
            .register_client_control_provider(key, Arc::new(factory))
    }

    pub fn register_channel(
        &mut self,
        key: &str,
        config: ChannelConfig,
    ) -> Result<ChannelId, ModRegistrationError> {
        self.backend.register_channel(key, config)
    }

    pub fn register_action_handler<H>(
        &mut self,
        action_kind: ActionKindId,
        handler: H,
    ) -> Result<(), ModRegistrationError>
    where
        H: ActionHandler + 'static,
    {
        self.backend
            .register_action_handler(action_kind, Box::new(handler))
    }

    pub fn register_action_kind(
        &mut self,
        key: &str,
    ) -> Result<ActionKindId, ModRegistrationError> {
        self.backend.register_action_kind(key)
    }

    pub fn on_start_client(&mut self, hook: StartClientHook) {
        self.backend.on_start_client(hook);
    }

    pub fn on_start_server(&mut self, hook: StartServerHook) {
        self.backend.on_start_server(hook);
    }

    pub fn on_tick_client(&mut self, hook: TickClientHook) {
        self.backend.on_tick_client(hook);
    }

    pub fn on_tick_server(&mut self, hook: TickServerHook) {
        self.backend.on_tick_server(hook);
    }

    pub fn on_client_messages(&mut self, hook: ClientMessagesHook) {
        self.backend.on_client_messages(hook);
    }

    pub fn on_server_messages(&mut self, hook: ServerMessagesHook) {
        self.backend.on_server_messages(hook);
    }
}

/// Numeric id for registered component keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u8);

/// Numeric id for registered component keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(pub u32);

/// Supported component codec contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ComponentCodec {
    /// Opaque bytes payload, interpreted by mod code.
    RawBytes,
}

/// Numeric id for registered message keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageId(pub u32);

/// Supported message codec contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MessageCodec {
    /// Opaque bytes payload, interpreted by higher-level mod code.
    RawBytes,
}

/// Message type registration config.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageConfig {
    pub codec: MessageCodec,
}

impl Default for MessageConfig {
    fn default() -> Self {
        Self {
            codec: MessageCodec::RawBytes,
        }
    }
}

/// Numeric id for registered worldgen providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldGenId(pub u32);

/// Numeric id for registered character controllers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CharacterControllerId(pub u32);

/// Numeric id for registered modnet channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelId(pub u32);

/// Numeric id for registered client control providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientControlProviderId(pub u32);

/// Error type for mod registration failures.
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum ModRegistrationError {
    #[error("duplicate {registry} key '{key}' registered by mod '{mod_id}'")]
    DuplicateKey {
        mod_id: String,
        registry: &'static str,
        key: String,
    },
    #[error("empty {registry} key registered by mod '{mod_id}'")]
    EmptyKey {
        mod_id: String,
        registry: &'static str,
    },
    #[error("too many blocks registered by mod '{mod_id}' for key '{key}': limit is {limit}")]
    TooManyBlocks {
        mod_id: String,
        key: String,
        limit: u32,
    },
    #[error(
        "unsupported channel QoS for mod '{mod_id}' key '{key}': reliability={reliability:?}, ordering={ordering:?}; v1 supports only (Reliable, Ordered) and (Unreliable, Unordered)"
    )]
    UnsupportedChannelQos {
        mod_id: String,
        key: String,
        reliability: ChannelReliability,
        ordering: ChannelOrdering,
    },
    #[error(
        "mod '{mod_id}' declared capability '{key}' but it is not present in the resolved capability table"
    )]
    UndeclaredCapability { mod_id: String, key: String },
    #[error("invalid {kind} declaration for mod '{mod_id}': {reason}")]
    InvalidDeclaration {
        mod_id: String,
        kind: &'static str,
        reason: String,
    },
}

/// Error type for mod config decode failures.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ModConfigError {
    #[error("failed to decode config for mod '{mod_id}'")]
    Deserialize {
        mod_id: String,
        #[source]
        source: toml::de::Error,
    },
}

/// Lifecycle callback executed once when the client side starts.
pub type StartClientHook = for<'a> fn(&mut ClientApi<'a>);

/// Lifecycle callback executed once when the server side starts.
pub type StartServerHook = for<'a> fn(&mut ServerApi<'a>);

/// Lifecycle callback executed on each client tick.
pub type TickClientHook = for<'a> fn(&mut ClientTickApi<'a>);

/// Lifecycle callback executed on each server tick.
pub type TickServerHook = for<'a> fn(&mut ServerTickApi<'a>);

/// Message callback executed on each client message dispatch phase.
pub type ClientMessagesHook = for<'a> fn(&mut ClientMessagesApi<'a>);

/// Message callback executed on each server message dispatch phase.
pub type ServerMessagesHook = for<'a> fn(&mut ServerMessagesApi<'a>);

/// Runtime-provided services exposed to SDK hooks.
pub trait Services {
    fn guest_runtime_service(
        &mut self,
        _request: &RuntimeServiceRequest,
    ) -> RuntimeServiceResponse {
        RuntimeServiceResponse::Unsupported
    }

    fn apply_guest_runtime_commands(
        &mut self,
        commands: &RuntimeCommandOutput,
    ) -> Result<(), RuntimeOutputApplyError> {
        if commands.is_empty() {
            Ok(())
        } else {
            Err(RuntimeOutputApplyError::UnsupportedFamily { family: "commands" })
        }
    }

    fn apply_guest_runtime_presentation(
        &mut self,
        presentation: &RuntimePresentationOutput,
    ) -> Result<(), RuntimeOutputApplyError> {
        if presentation.is_empty() {
            Ok(())
        } else {
            Err(RuntimeOutputApplyError::UnsupportedFamily {
                family: "presentation",
            })
        }
    }

    fn record_guest_log(&mut self, _record: &HostLogRecord) {}
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum RuntimeOutputApplyError {
    #[error("runtime output family '{family}' is not supported in this host context")]
    UnsupportedFamily { family: &'static str },
    #[error("runtime output application failed: {message}")]
    Rejected { message: String },
}

/// Empty services implementation used by runtimes that do not expose services yet.
#[derive(Debug, Default)]
pub struct NoServices;

impl Services for NoServices {}

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
    pub predicted_block_id: u8,
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
    pub block_id: u8,
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

/// Nameplate draw command (screen-space).
#[derive(Debug, Clone, PartialEq)]
pub struct ClientNameplateDrawCmd {
    pub text: String,
    pub screen_pos_px: (f32, f32),
    pub rgba: (u8, u8, u8, u8),
}

/// Engine-provided client input surface.
pub trait ClientInputProvider {
    fn mouse_button_down(&self, button: ClientMouseButton) -> bool;
    fn mouse_button_just_pressed(&self, button: ClientMouseButton) -> bool;
    fn key_down(&self, key: ClientKeyCode) -> bool;
    fn key_just_pressed(&self, key: ClientKeyCode) -> bool;
    fn bind_mouse_button(&mut self, button: ClientMouseButton, owner: &str) -> bool;
    fn bind_key(&mut self, key: ClientKeyCode, owner: &str) -> bool;
    fn consume_mouse_button_press(&mut self, button: ClientMouseButton, owner: &str) -> bool;
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
    fn cursor_hit(&self, max_distance_m: f32) -> Option<ClientCursorHit>;
    fn block_id_at(&self, pos: (i32, i32, i32)) -> Option<u8>;
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
    fn submit_action(&mut self, req: ClientActionRequest) -> Result<u32, ClientActionSubmitError>;

    fn poll_action_result(&mut self) -> Option<ClientActionResultEvent>;
}

/// Mod message scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MessageScope {
    Global,
    Level { level_id: u32, stream_epoch: u32 },
}

/// Client outbound scope selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ClientOutboundMessageScope {
    Global,
    ActiveLevel,
}

/// Outbound client mod message payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientOutboundMessage {
    pub scope: ClientOutboundMessageScope,
    pub channel_id: u32,
    pub message_id: u32,
    pub seq: Option<u32>,
    pub payload: Vec<u8>,
}

/// Inbound client mod message payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientInboundMessage {
    pub scope: MessageScope,
    pub channel_id: u32,
    pub message_id: u32,
    pub seq: Option<u32>,
    pub payload: Vec<u8>,
}

/// Outbound server mod message payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerOutboundMessage {
    pub scope: MessageScope,
    pub channel_id: u32,
    pub message_id: u32,
    pub seq: Option<u32>,
    pub payload: Vec<u8>,
}

/// Inbound server mod message payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerInboundMessage {
    pub player_id: u64,
    pub scope: MessageScope,
    pub channel_id: u32,
    pub message_id: u32,
    pub seq: Option<u32>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("client message send failed: {message}")]
pub struct ClientMessageSendError {
    pub message: String,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("server message send failed: {message}")]
pub struct ServerMessageSendError {
    pub message: String,
}

/// Engine-provided client mod message surface.
pub trait ClientMessageProvider {
    fn send_msg(&mut self, msg: ClientOutboundMessage) -> Result<(), ClientMessageSendError>;
    fn poll_msg(&mut self) -> Option<ClientInboundMessage>;
}

/// Engine-provided server mod message surface.
pub trait ServerMessageProvider {
    fn send_to(
        &mut self,
        player_id: u64,
        msg: ServerOutboundMessage,
    ) -> Result<(), ServerMessageSendError>;
    fn poll_msg(&mut self) -> Option<ServerInboundMessage>;
}

/// Engine-provided client message send surface.
pub trait ClientMessageSender {
    fn send_msg(&mut self, msg: ClientOutboundMessage) -> Result<(), ClientMessageSendError>;
}

impl<T> ClientMessageSender for T
where
    T: ClientMessageProvider + ?Sized,
{
    fn send_msg(&mut self, msg: ClientOutboundMessage) -> Result<(), ClientMessageSendError> {
        ClientMessageProvider::send_msg(self, msg)
    }
}

/// Engine-provided server message send surface.
pub trait ServerMessageSender {
    fn send_to(
        &mut self,
        player_id: u64,
        msg: ServerOutboundMessage,
    ) -> Result<(), ServerMessageSendError>;
}

impl<T> ServerMessageSender for T
where
    T: ServerMessageProvider + ?Sized,
{
    fn send_to(
        &mut self,
        player_id: u64,
        msg: ServerOutboundMessage,
    ) -> Result<(), ServerMessageSendError> {
        ServerMessageProvider::send_to(self, player_id, msg)
    }
}

/// Engine-provided player presentation query surface.
pub trait ClientPlayerProvider {
    fn list_players(&self, out: &mut Vec<ClientPlayerView>);
    fn display_name_for(&self, player_id: u64) -> Option<String>;
    fn component_bytes_for(&self, player_id: u64, component_id: ComponentId) -> Option<&[u8]>;
    fn world_to_screen(&self, world_pos_m: (f32, f32, f32)) -> Option<(f32, f32)>;
}

/// Engine-owned queue for nameplate draw commands.
pub trait ClientNameplateProvider {
    fn clear_nameplates(&mut self);
    fn push_nameplate(&mut self, cmd: ClientNameplateDrawCmd);
}

/// Server-side lifecycle API.
pub struct ServerApi<'a> {
    pub services: &'a mut dyn Services,
}

impl<'a> ServerApi<'a> {
    #[must_use]
    pub fn new(services: &'a mut dyn Services) -> Self {
        Self { services }
    }

    pub fn log(&mut self, level: LogLevel, message: impl AsRef<str>) {
        let _ = &self.services;
        emit_log(level, message);
    }
}

/// Client-side lifecycle API.
pub struct ClientApi<'a> {
    pub services: &'a mut dyn Services,
    pub input: &'a mut dyn ClientInputProvider,
    pub camera: &'a mut dyn ClientCameraHitProvider,
    pub interaction: &'a mut dyn ClientInteractionProvider,
    pub players: &'a mut dyn ClientPlayerProvider,
    pub nameplates: &'a mut dyn ClientNameplateProvider,
}

impl<'a> ClientApi<'a> {
    #[must_use]
    pub fn new(
        services: &'a mut dyn Services,
        input: &'a mut dyn ClientInputProvider,
        camera: &'a mut dyn ClientCameraHitProvider,
        interaction: &'a mut dyn ClientInteractionProvider,
        players: &'a mut dyn ClientPlayerProvider,
        nameplates: &'a mut dyn ClientNameplateProvider,
    ) -> Self {
        Self {
            services,
            input,
            camera,
            interaction,
            players,
            nameplates,
        }
    }

    #[must_use]
    pub fn reborrow(&mut self) -> ClientApi<'_> {
        ClientApi {
            services: self.services,
            input: self.input,
            camera: self.camera,
            interaction: self.interaction,
            players: self.players,
            nameplates: self.nameplates,
        }
    }

    pub fn log(&mut self, level: LogLevel, message: impl AsRef<str>) {
        let _ = &self.services;
        emit_log(level, message);
    }
}

/// Client-side message dispatch context.
pub struct ClientMessagesApi<'a> {
    pub tick: u64,
    pub dt: Duration,
    pub services: &'a mut dyn Services,
    pub inbound: &'a [ClientInboundMessage],
    pub sender: &'a mut dyn ClientMessageSender,
}

impl<'a> ClientMessagesApi<'a> {
    #[must_use]
    pub fn new(
        tick: u64,
        dt: Duration,
        services: &'a mut dyn Services,
        inbound: &'a [ClientInboundMessage],
        sender: &'a mut dyn ClientMessageSender,
    ) -> Self {
        Self {
            tick,
            dt,
            services,
            inbound,
            sender,
        }
    }

    pub fn log(&mut self, level: LogLevel, message: impl AsRef<str>) {
        let _ = &self.services;
        emit_log(level, message);
    }
}

/// Server-side message dispatch context.
pub struct ServerMessagesApi<'a> {
    pub tick: u64,
    pub dt: Duration,
    pub services: &'a mut dyn Services,
    pub inbound: &'a [ServerInboundMessage],
    pub sender: &'a mut dyn ServerMessageSender,
}

impl<'a> ServerMessagesApi<'a> {
    #[must_use]
    pub fn new(
        tick: u64,
        dt: Duration,
        services: &'a mut dyn Services,
        inbound: &'a [ServerInboundMessage],
        sender: &'a mut dyn ServerMessageSender,
    ) -> Self {
        Self {
            tick,
            dt,
            services,
            inbound,
            sender,
        }
    }

    pub fn log(&mut self, level: LogLevel, message: impl AsRef<str>) {
        let _ = &self.services;
        emit_log(level, message);
    }
}

/// Client-side lifecycle tick context.
pub struct ClientTickApi<'a> {
    pub tick: u64,
    pub dt: Duration,
    pub client: ClientApi<'a>,
}

/// Client control provider output for one input sample.
///
/// Notes:
/// - The engine owns input sequencing (`NetSeq`) as part of the prediction/network timeline.
/// - Control providers must NOT generate or persist input sequence numbers.
#[derive(Debug, Clone)]
pub struct ClientControlOutput {
    pub input: Arc<[u8]>,
    pub view_yaw_deg: f32,
    pub view_pitch_deg: f32,
}

/// Timeline metadata associated with one controller input sample.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InputTimeline {
    pub input_seq: u32,
    pub sim_tick: u64,
}

/// Opaque controller input consumed by character controllers.
#[derive(Debug, Clone)]
pub struct CharacterControllerInput {
    pub input: Arc<[u8]>,
    pub view_yaw_deg: f32,
    pub view_pitch_deg: f32,
    pub timeline: InputTimeline,
}

/// Init params for client control provider factories.
///
/// Reserved for future evolution (e.g., default sensitivity presets).
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct ClientControlProviderInit {}

/// Contract for gameplay control providers owned by mods.
///
/// This is a pure mapping: device state -> raw input.
/// Providers may keep internal filters (e.g. smoothing), but must not own network sequencing.
pub trait ClientControlProvider: Send + Sync {
    fn sample(&mut self, device: &mut dyn ClientControlDeviceState) -> ClientControlOutput;

    /// Optional hook to clear internal filters on hard resets (world barrier / reconnect).
    fn reset(&mut self) {}
}

impl<'a> ClientTickApi<'a> {
    #[must_use]
    pub fn new(tick: u64, dt: Duration, client: ClientApi<'a>) -> Self {
        Self { tick, dt, client }
    }

    pub fn log(&mut self, level: LogLevel, message: impl AsRef<str>) {
        self.client.log(level, message);
    }
}

/// Server-side lifecycle tick context.
pub struct ServerTickApi<'a> {
    pub tick: u64,
    pub dt: Duration,
    pub server: ServerApi<'a>,
}

impl<'a> ServerTickApi<'a> {
    #[must_use]
    pub fn new(tick: u64, dt: Duration, server: ServerApi<'a>) -> Self {
        Self { tick, dt, server }
    }

    pub fn log(&mut self, level: LogLevel, message: impl AsRef<str>) {
        self.server.log(level, message);
    }
}

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
}

impl WorldGenInit {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            world_id: None,
        }
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

/// Generated section payload for one vertical section in a column.
#[derive(Debug, Clone)]
pub struct WorldGenSection {
    pub sy: i8,
    pub blocks: Vec<u8>,
}

/// Minimal worldgen output contract.
#[derive(Debug, Default, Clone)]
pub struct WorldGenOutput {
    pub sections: Vec<WorldGenSection>,
}

/// Worldgen contract error placeholder.
#[derive(Debug, thiserror::Error)]
#[error("worldgen error: {message}")]
pub struct WorldGenError {
    pub message: String,
}

/// Character shape used for collision queries.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum CharacterShape {
    Aabb { half_extents: [f32; 3] },
}

/// Character controller configuration.
#[derive(Debug, Clone, Copy)]
pub struct CharacterConfig {
    pub shape: CharacterShape,
    pub max_speed_ground: f32,
    pub max_speed_air: f32,
    pub accel_ground: f32,
    pub accel_air: f32,
    pub gravity: f32,
    pub jump_impulse: f32,
    /// Maximum step-up height in meters.
    ///
    /// This is **controller-defined** behavior: the engine does not apply stepping by itself.
    /// Controllers may use this value to implement classic "step-up" (walk up small ledges)
    /// using additional collision probes/resolution.
    ///
    /// MVP note:
    /// - `freven_vanilla_essentials` humanoid controller currently does not implement step-up
    ///   and keeps `step_height = 0.0`.
    pub step_height: f32,
    pub skin_width: f32,
}

/// Runtime state stepped by a character controller.
#[derive(Debug, Clone, Copy)]
pub struct CharacterState {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub on_ground: bool,
}

/// Wire millimeter scale used for position/velocity quantization.
pub const WIRE_MM_SCALE: f32 = 1000.0;

/// Quantize meters to wire millimeters using round-to-nearest.
#[inline]
#[must_use]
pub fn quantize_mm_i32(value_m: f32) -> i32 {
    let mm = (value_m * WIRE_MM_SCALE).round();
    mm.clamp(i32::MIN as f32, i32::MAX as f32) as i32
}

/// Dequantize wire millimeters back to meters.
#[inline]
#[must_use]
pub fn dequantize_mm_i32(value_mm: i32) -> f32 {
    value_mm as f32 / WIRE_MM_SCALE
}

/// Round-trip meters through wire millimeter precision.
#[inline]
#[must_use]
pub fn quantize_m_to_wire_mm(value_m: f32) -> f32 {
    dequantize_mm_i32(quantize_mm_i32(value_m))
}

/// Quantize character runtime state to wire millimeter precision.
#[inline]
pub fn quantize_character_state_mm(state: &mut CharacterState) {
    state.pos[0] = quantize_m_to_wire_mm(state.pos[0]);
    state.pos[1] = quantize_m_to_wire_mm(state.pos[1]);
    state.pos[2] = quantize_m_to_wire_mm(state.pos[2]);
    state.vel[0] = quantize_m_to_wire_mm(state.vel[0]);
    state.vel[1] = quantize_m_to_wire_mm(state.vel[1]);
    state.vel[2] = quantize_m_to_wire_mm(state.vel[2]);
}

/// Sweep query result for AABB movement.
#[derive(Debug, Clone, Copy)]
pub struct SweepHit {
    pub hit: bool,
    pub toi: f32,
    pub normal: [f32; 3],
}

impl Default for SweepHit {
    fn default() -> Self {
        Self {
            hit: false,
            toi: 1.0,
            normal: [0.0, 0.0, 0.0],
        }
    }
}

/// Terrain solidity sample for kinematic AABB movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolidSample {
    /// True when sampled voxel is solid.
    pub solid: bool,
    /// True when voxel state is known/loaded.
    pub known: bool,
}

impl SolidSample {
    /// Known sample constructor.
    #[must_use]
    pub const fn known(solid: bool) -> Self {
        Self { solid, known: true }
    }

    /// Unknown sample constructor.
    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            solid: true,
            known: false,
        }
    }
}

/// Configuration for deterministic kinematic terrain movement.
#[derive(Debug, Clone, Copy)]
pub struct KinematicMoveConfig {
    /// Desired wall/floor gap in meters.
    pub skin_width: f32,
    /// Tiny numerical epsilon used only for overlap/range stability.
    pub contact_epsilon: f32,
    /// Upper bound on internal substeps used for large motions.
    pub max_substeps: u8,
    /// Maximum absolute axis motion per internal substep.
    pub max_motion_per_step: f32,
}

impl KinematicMoveConfig {
    const SKIN_MIN: f32 = 1.0e-5;
    const SKIN_MAX: f32 = 2.0e-2;
    const EPS_MIN: f32 = 1.0e-6;
    const EPS_MAX: f32 = 1.0e-3;
    const MAX_SUBSTEPS_MIN: u8 = 1;
    const MAX_SUBSTEPS_MAX: u8 = 16;
    const MOTION_STEP_MIN: f32 = 1.0e-3;
    const MOTION_STEP_MAX: f32 = 10.0;

    /// Return a clamped config suitable for simulation/runtime use.
    #[must_use]
    pub fn validated(mut self) -> Self {
        self.skin_width = self.skin_width.abs().clamp(Self::SKIN_MIN, Self::SKIN_MAX);
        self.contact_epsilon = self
            .contact_epsilon
            .abs()
            .clamp(Self::EPS_MIN, Self::EPS_MAX);
        self.max_substeps = self
            .max_substeps
            .clamp(Self::MAX_SUBSTEPS_MIN, Self::MAX_SUBSTEPS_MAX);
        self.max_motion_per_step = self
            .max_motion_per_step
            .abs()
            .clamp(Self::MOTION_STEP_MIN, Self::MOTION_STEP_MAX);
        self
    }
}

impl Default for KinematicMoveConfig {
    fn default() -> Self {
        Self {
            skin_width: 0.001,
            contact_epsilon: 1.0e-4,
            max_substeps: 4,
            max_motion_per_step: 0.75,
        }
    }
}

/// Result for deterministic kinematic terrain movement.
#[derive(Debug, Clone, Copy)]
pub struct KinematicMoveResult {
    pub pos: [f32; 3],
    pub applied_motion: [f32; 3],
    pub hit_x: bool,
    pub hit_y: bool,
    pub hit_z: bool,
    pub hit_ground: bool,
    pub started_overlapping: bool,
    pub collision_incomplete: bool,
}

impl Default for KinematicMoveResult {
    fn default() -> Self {
        Self {
            pos: [0.0, 0.0, 0.0],
            applied_motion: [0.0, 0.0, 0.0],
            hit_x: false,
            hit_y: false,
            hit_z: false,
            hit_ground: false,
            started_overlapping: false,
            collision_incomplete: false,
        }
    }
}

/// Engine-side collision queries consumed by character controllers.
pub trait CharacterPhysics {
    fn is_solid_world_collision(&mut self, wx: i32, wy: i32, wz: i32) -> bool;
    fn sweep_aabb(&mut self, half_extents: [f32; 3], from: [f32; 3], to: [f32; 3]) -> SweepHit;
    fn move_aabb_terrain(
        &mut self,
        half_extents: [f32; 3],
        pos: [f32; 3],
        motion: [f32; 3],
        cfg: KinematicMoveConfig,
    ) -> KinematicMoveResult;
}

/// Character controller trait used for authoritative movement and prediction.
pub trait CharacterController: Send + Sync {
    fn config(&self) -> &CharacterConfig;
    fn step(
        &mut self,
        state: &mut CharacterState,
        input: &CharacterControllerInput,
        physics: &mut dyn CharacterPhysics,
        dt: Duration,
    );
}

/// Character controller factory init parameters.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct CharacterControllerInit {}

/// Character controller factory.
pub type CharacterControllerFactory =
    Arc<dyn Fn(CharacterControllerInit) -> Box<dyn CharacterController> + Send + Sync>;

/// Client control provider factory.
pub type ClientControlProviderFactory =
    Arc<dyn Fn(ClientControlProviderInit) -> Box<dyn ClientControlProvider> + Send + Sync>;

/// Channel reliability policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ChannelReliability {
    Reliable,
    Unreliable,
}

/// Channel ordering policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ChannelOrdering {
    Ordered,
    Unordered,
}

/// Channel traffic direction policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ChannelDirection {
    ClientToServer,
    ServerToClient,
    Bidirectional,
}

/// Optional per-channel budget contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChannelBudget {
    pub max_messages_per_sec: Option<u32>,
    pub max_bytes_per_sec: Option<u32>,
}

/// ModNet channel contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelConfig {
    pub reliability: ChannelReliability,
    pub ordering: ChannelOrdering,
    pub direction: ChannelDirection,
    pub budget: Option<ChannelBudget>,
}

#[doc(hidden)]
pub mod __private {
    use super::{OBSERVABILITY_BRIDGE, ObservabilityBridge};

    pub fn with_observability_bridge<T>(
        bridge: ObservabilityBridge,
        call: impl FnOnce() -> T,
    ) -> T {
        OBSERVABILITY_BRIDGE.with(|slot| {
            let previous = slot.replace(bridge);
            let result = call();
            slot.replace(previous);
            result
        })
    }
}
