//! Canonical public guest contract for runtime-loaded Freven mods.
//!
//! The crate is transport-agnostic by design. Wasm ptr/len exports, native
//! process envelopes, and other backend-specific details live in transport
//! crates and docs, not in the semantic contract.

extern crate alloc;

use alloc::{string::String, vec::Vec};
use freven_sdk_types::blocks::BlockDef;
use serde::{Deserialize, Serialize};

pub const GUEST_CONTRACT_VERSION_1: u32 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GuestTransport {
    WasmPtrLenV1,
    NativeInProcessV1,
    ExternalEnvelopeV1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeGuestInput {
    pub ptr: *const u8,
    pub len: usize,
}

impl NativeGuestInput {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            ptr: core::ptr::null(),
            len: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeGuestBuffer {
    pub ptr: *mut u8,
    pub len: usize,
}

impl NativeGuestBuffer {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            ptr: core::ptr::null_mut(),
            len: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NegotiationRequest {
    pub supported_contract_versions: Vec<u32>,
    pub transport: GuestTransport,
}

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
    pub worldgen: Vec<WorldGenDeclaration>,
    pub character_controllers: Vec<CharacterControllerDeclaration>,
    pub client_control_providers: Vec<ClientControlProviderDeclaration>,
    pub channels: Vec<ChannelDeclaration>,
    pub actions: Vec<ActionDeclaration>,
    pub capabilities: Vec<CapabilityDeclaration>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuestCallbacks {
    pub lifecycle: LifecycleHooks,
    pub action: bool,
    pub messages: MessageHooks,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LifecycleHooks {
    pub start_client: bool,
    pub start_server: bool,
    pub tick_client: bool,
    pub tick_server: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageHooks {
    pub client: bool,
    pub server: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDeclaration {
    pub key: String,
    pub def: BlockDef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComponentDeclaration {
    pub key: String,
    pub codec: ComponentCodec,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComponentCodec {
    RawBytes,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageDeclaration {
    pub key: String,
    pub codec: MessageCodec,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageCodec {
    RawBytes,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChannelDeclaration {
    pub key: String,
    pub config: ChannelConfig,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChannelConfig {
    pub reliability: ChannelReliability,
    pub ordering: ChannelOrdering,
    pub direction: ChannelDirection,
    pub budget: Option<ChannelBudget>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChannelReliability {
    Reliable,
    Unreliable,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChannelOrdering {
    Ordered,
    Unordered,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChannelDirection {
    ClientToServer,
    ServerToClient,
    Bidirectional,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChannelBudget {
    pub max_messages_per_sec: Option<u32>,
    pub max_bytes_per_sec: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionDeclaration {
    pub key: String,
    pub binding_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityDeclaration {
    pub key: String,
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
    pub player_position_m: Option<[f32; 3]>,
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
pub struct LifecycleAck {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionResult {
    pub outcome: ActionOutcome,
    pub effects: EffectBatch,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ClientMessageResult {
    pub outbound: Vec<ClientOutboundMessage>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ServerMessageResult {
    pub outbound: Vec<ServerOutboundMessage>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionOutcome {
    Applied,
    Rejected,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct EffectBatch {
    pub world: Vec<WorldEffect>,
}

impl EffectBatch {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.world.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorldEffect {
    SetBlock { pos: (i32, i32, i32), block_id: u8 },
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
