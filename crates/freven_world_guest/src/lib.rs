//! Canonical public guest contract for runtime-loaded Freven mods.
//!
//! The crate is transport-agnostic by design. Wasm ptr/len exports, native
//! process envelopes, and other backend-specific details live in transport
//! crates and docs, not in the semantic contract.
//!
//! Ownership boundaries:
//! - generic guest/runtime transport semantics live in `freven_guest`
//! - volumetric topology/addressing live in `freven_volumetric_sdk_types`
//! - standard block/profile vocabulary lives in `freven_block_sdk_types`
//! - this crate defines the canonical runtime-loaded world guest contract that
//!   consumes those lower-layer vocabularies
//!
//! Composition note:
//! - block/profile vocabulary is owned by `freven_block_sdk_types`
//! - runtime-loaded block mutation/query/service contracts are owned by
//!   `freven_block_guest`
//! - this crate may still carry block-owned families inside the generic
//!   world guest/runtime envelope
//! - that carrier role does not make `freven_world_guest` the owner of
//!   block gameplay semantics

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::{string::String, vec::Vec};
use freven_block_guest::{BlockMutationBatch, BlockServiceRequest, BlockServiceResponse};
use freven_block_sdk_types::{BlockDescriptor, BlockRuntimeId};
use freven_guest::{
    CapabilityDeclaration, ChannelDeclaration, ComponentDeclaration, LifecycleHooks, LogPayload,
    MessageDeclaration, MessageHooks, RuntimeSessionInfo,
};
use freven_volumetric_sdk_types::{ColumnCoord, SectionY, WorldCellPos};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationResponse {
    pub selected_contract_version: u32,
    pub description: GuestDescription,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestDescription {
    pub guest_id: String,
    pub registration: GuestRegistration,
    pub callbacks: GuestCallbacks,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GuestRegistration {
    pub blocks: Vec<BlockDeclaration>,
    pub components: Vec<ComponentDeclaration>,
    pub messages: Vec<MessageDeclaration>,
    pub world: WorldGuestRegistration,
    pub avatar: AvatarGuestRegistration,
    pub channels: Vec<ChannelDeclaration>,
    pub actions: Vec<ActionDeclaration>,
    pub capabilities: Vec<CapabilityDeclaration>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldGuestRegistration {
    pub worldgen: Vec<WorldGenDeclaration>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AvatarGuestRegistration {
    pub character_controllers: Vec<CharacterControllerDeclaration>,
    pub client_control_providers: Vec<ClientControlProviderDeclaration>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuestCallbacks {
    pub lifecycle: LifecycleHooks,
    pub action: bool,
    pub messages: MessageHooks,
    pub providers: ProviderHooks,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderHooks {
    pub world: WorldProviderHooks,
    pub avatar: AvatarProviderHooks,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldProviderHooks {
    pub worldgen: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AvatarProviderHooks {
    pub character_controller: bool,
    pub client_control_provider: bool,
}

/// Runtime-loaded declaration of a reusable standard block/profile entry.
///
/// `BlockDescriptor` is imported from `freven_block_sdk_types`, which owns the
/// public standard block/profile vocabulary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDeclaration {
    pub key: String,
    pub def: BlockDescriptor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldGenDeclaration {
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterControllerDeclaration {
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientControlProviderDeclaration {
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionDeclaration {
    pub key: String,
    pub binding_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldGenCallInput {
    pub key: String,
    pub init: WorldGenInit,
    pub request: WorldGenRequest,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct WorldGenCallResult {
    pub output: WorldGenOutput,
}

/// Worldgen init payload for a runtime-loaded guest.
///
/// Volumetric topology/addressing live in `freven_volumetric_sdk_types`.
/// Standard block/profile ids are imported from `freven_block_sdk_types`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct WorldGenInit {
    pub seed: u64,
    pub world_id: Option<String>,
    /// Stable string-key -> standard block/profile runtime id mapping.
    pub block_ids: BTreeMap<String, BlockRuntimeId>,
}

impl WorldGenInit {
    #[must_use]
    pub fn block_id_by_key(&self, key: &str) -> Option<BlockRuntimeId> {
        self.block_ids.get(key).copied()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct WorldGenOutput {
    pub writes: Vec<WorldTerrainWrite>,
}

/// Terrain writes emitted by a runtime-loaded worldgen provider.
///
/// Volumetric addressing is owned by `freven_volumetric_sdk_types`.
/// Standard block/profile ids are imported from `freven_block_sdk_types`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CharacterControllerInitInput {
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CharacterControllerInitResult {
    pub config: CharacterConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CharacterControllerStepInput {
    pub key: String,
    pub state: CharacterState,
    pub input: CharacterControllerInput,
    pub dt_millis: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CharacterControllerStepResult {
    pub state: CharacterState,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CharacterShape {
    Aabb { half_extents: [f32; 3] },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct CharacterConfig {
    pub shape: CharacterShape,
    pub max_speed_ground: f32,
    pub max_speed_air: f32,
    pub accel_ground: f32,
    pub accel_air: f32,
    pub gravity: f32,
    pub jump_impulse: f32,
    pub step_height: f32,
    pub skin_width: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct CharacterState {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub on_ground: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterControllerInput {
    pub input: Vec<u8>,
    pub view_yaw_deg_mdeg: i32,
    pub view_pitch_deg_mdeg: i32,
    pub timeline: InputTimeline,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputTimeline {
    pub input_seq: u32,
    pub sim_tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientControlSampleInput {
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientControlSampleResult {
    pub output: ClientControlOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientControlOutput {
    pub input: Vec<u8>,
    pub view_yaw_deg_mdeg: i32,
    pub view_pitch_deg_mdeg: i32,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModConfigFormat {
    #[default]
    Toml,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ModConfigDocument {
    pub format: ModConfigFormat,
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct StartInput {
    pub session: RuntimeSessionInfo,
    pub experience_id: String,
    pub mod_id: String,
    pub config: ModConfigDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TickInput {
    pub tick: u64,
    pub dt_millis: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionInput<'a> {
    pub binding_id: u32,
    pub player_id: u64,
    pub level_id: u32,
    pub stream_epoch: u32,
    pub action_seq: u32,
    pub at_input_seq: u32,
    #[serde(borrow)]
    pub payload: &'a [u8],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientMessageInput {
    pub tick: u64,
    pub dt_millis: u32,
    pub messages: Vec<ClientInboundMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerMessageInput {
    pub tick: u64,
    pub dt_millis: u32,
    pub messages: Vec<ServerInboundMessage>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct LifecycleResult {
    pub output: RuntimeOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionResult {
    pub outcome: ActionOutcome,
    pub output: RuntimeOutput,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ClientMessageResult {
    pub output: RuntimeOutput,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ServerMessageResult {
    pub output: RuntimeOutput,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionOutcome {
    Applied,
    Rejected,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RuntimeOutput {
    pub messages: RuntimeMessageOutput,
    pub blocks: BlockMutationBatch,
}

impl RuntimeOutput {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty() && self.blocks.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RuntimeMessageOutput {
    pub client: Vec<ClientOutboundMessage>,
    pub server: Vec<ServerOutboundMessage>,
}

impl RuntimeMessageOutput {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.client.is_empty() && self.server.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientOutboundMessage {
    pub scope: ClientOutboundMessageScope,
    pub channel_id: u32,
    pub message_id: u32,
    pub seq: Option<u32>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientInboundMessage {
    pub scope: MessageScope,
    pub channel_id: u32,
    pub message_id: u32,
    pub seq: Option<u32>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerOutboundMessage {
    pub player_id: u64,
    pub scope: MessageScope,
    pub channel_id: u32,
    pub message_id: u32,
    pub seq: Option<u32>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerInboundMessage {
    pub player_id: u64,
    pub scope: MessageScope,
    pub channel_id: u32,
    pub message_id: u32,
    pub seq: Option<u32>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageScope {
    Global,
    Level { level_id: u32, stream_epoch: u32 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClientOutboundMessageScope {
    Global,
    ActiveLevel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeLevelRef {
    pub level_id: u32,
    pub stream_epoch: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ClientPlayerView {
    pub player_id: u64,
    pub world_pos_m: (f32, f32, f32),
    pub is_local: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClientMouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct KinematicMoveConfig {
    pub skin_width: f32,
    pub contact_epsilon: f32,
    pub max_substeps: u8,
    pub max_motion_per_step: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SweepHit {
    pub hit: bool,
    pub toi: f32,
    pub normal: [f32; 3],
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeEntityTarget {
    Player { player_id: u64 },
    Entity { entity_id: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorldQueryRequest {
    PlayerPosition { player_id: u64 },
    PlayerDisplayName { player_id: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorldQueryResponse {
    PlayerPosition(Option<[f32; 3]>),
    PlayerDisplayName(Option<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClientVisibilityRequest {
    ClientPlayerViews,
    ClientWorldToScreen { world_pos_m: (f32, f32, f32) },
    ClientActiveLevel,
    ClientNextInputSeq,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClientVisibilityResponse {
    ClientPlayerViews(Vec<ClientPlayerView>),
    ClientWorldToScreen(Option<(i32, i32)>),
    ClientActiveLevel(Option<RuntimeLevelRef>),
    ClientNextInputSeq(Option<u32>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorldSessionRequest {
    ServerPlayerConnected { player_id: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorldSessionResponse {
    ServerPlayerConnected(Option<bool>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeClientControlRequest {
    BindMouseButton {
        button: ClientMouseButton,
        owner: String,
    },
    BindKey {
        key: ClientKeyCode,
        owner: String,
    },
    MouseButtonDown {
        button: ClientMouseButton,
        owner: String,
    },
    KeyDown {
        key: ClientKeyCode,
        owner: String,
    },
    MouseDelta,
    CursorLocked,
    ViewAnglesDegMdeg,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuntimeCharacterPhysicsRequest {
    IsSolidWorldCollision {
        wx: i32,
        wy: i32,
        wz: i32,
    },
    SweepAabb {
        half_extents: [f32; 3],
        from: [f32; 3],
        to: [f32; 3],
    },
    MoveAabbTerrain {
        half_extents: [f32; 3],
        pos: [f32; 3],
        motion: [f32; 3],
        cfg: KinematicMoveConfig,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeObservabilityRequest {
    Log(LogPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorldServiceRequest {
    Block(BlockServiceRequest),
    Query(WorldQueryRequest),
    ClientVisibility(ClientVisibilityRequest),
    Session(WorldSessionRequest),
    ClientControl(RuntimeClientControlRequest),
    CharacterPhysics(RuntimeCharacterPhysicsRequest),
    Observability(RuntimeObservabilityRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorldServiceResponse {
    Block(BlockServiceResponse),
    Query(WorldQueryResponse),
    ClientVisibility(ClientVisibilityResponse),
    Session(WorldSessionResponse),
    ClientControlBool(bool),
    ClientControlMouseDelta((i32, i32)),
    ClientControlViewAnglesDegMdeg((i32, i32)),
    CharacterPhysicsIsSolidWorldCollision(bool),
    CharacterPhysicsSweepAabb(SweepHit),
    CharacterPhysicsMoveAabbTerrain(KinematicMoveResult),
    Unsupported,
}
