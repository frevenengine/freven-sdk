//! Canonical public guest contract for runtime-loaded Freven mods.
//!
//! The crate is transport-agnostic by design. Wasm ptr/len exports, native
//! process envelopes, and other backend-specific details live in transport
//! crates and docs, not in the semantic contract.

extern crate alloc;

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

pub const GUEST_CONTRACT_VERSION_1: u32 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GuestTransport {
    WasmPtrLenV1,
    NativePtrLenV1,
    ExternalEnvelopeV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NegotiationRequest {
    pub supported_contract_versions: Vec<u32>,
    pub transport: GuestTransport,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NegotiationResponse {
    pub selected_contract_version: u32,
    pub description: GuestDescription,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuestDescription {
    pub guest_id: String,
    pub lifecycle: LifecycleHooks,
    pub action_entrypoint: bool,
    pub actions: Vec<ActionBinding>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LifecycleHooks {
    pub start_client: bool,
    pub start_server: bool,
    pub tick_client: bool,
    pub tick_server: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionBinding {
    pub key: String,
    pub binding_id: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct StartInput {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TickInput {
    pub tick: u64,
    pub dt_millis: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LifecycleAck {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionResult {
    pub outcome: ActionOutcome,
    pub effects: EffectBatch,
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
