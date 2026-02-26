//! Stable SDK contracts for Freven experiences and compile-time mods.
//!
//! Responsibilities:
//! - define experience/mod descriptors used by boot/runtime layers
//! - expose deterministic registration surfaces (components/messages/worldgen/modnet)
//! - define stable hook contexts and registration errors
//!
//! Extension guidance:
//! - add new registries behind stable string keys
//! - keep hook/context types engine-agnostic
//! - avoid leaking runtime/transport implementation details

use std::time::Duration;

use freven_core::blocks::BlockDef;
use serde::de::DeserializeOwned;

pub mod action_payloads;

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

/// Temporary action kind for legacy block-break semantics.
pub const ACTION_KIND_BLOCK_BREAK: ActionKindId = ActionKindId(1);
/// Temporary action kind for legacy block-place semantics.
pub const ACTION_KIND_BLOCK_PLACE: ActionKindId = ActionKindId(2);

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

/// Server-authoritative world-edit service exposed to action handlers.
pub trait ActionWorldEdit {}

/// Character-physics query service exposed to action handlers.
pub trait CharacterPhysicsQuery {
    fn player_position(&self, player_id: u64) -> Option<[f32; 3]>;
}

/// Stable action-dispatch context provided by runtime/server integration.
pub struct ActionContext<'a> {
    pub world_read: Option<&'a dyn ActionWorldRead>,
    pub world_edit: Option<&'a mut dyn ActionWorldEdit>,
    pub character_physics: Option<&'a dyn CharacterPhysicsQuery>,
    pub player_id: u64,
    pub at_input_seq: u32,
}

impl<'a> ActionContext<'a> {
    #[must_use]
    pub fn new(
        world_read: Option<&'a dyn ActionWorldRead>,
        world_edit: Option<&'a mut dyn ActionWorldEdit>,
        character_physics: Option<&'a dyn CharacterPhysicsQuery>,
        player_id: u64,
        at_input_seq: u32,
    ) -> Self {
        Self {
            world_read,
            world_edit,
            character_physics,
            player_id,
            at_input_seq,
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

/// Experience specification selected by boot.
///
/// `config` is a top-level table keyed by mod id. Each mod receives its own value.
#[derive(Clone)]
pub struct ExperienceSpec {
    pub id: String,
    pub mods: Vec<ModDescriptor>,
    pub default_worldgen: Option<String>,
    pub default_character_controller: Option<String>,
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
    fn register_component(&mut self, key: &str) -> Result<ComponentId, ModRegistrationError>;
    fn register_message(&mut self, key: &str) -> Result<MessageId, ModRegistrationError>;
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
    fn on_server_tick(&mut self, hook: ServerTickHook);
    fn on_client_tick(&mut self, hook: ClientTickHook);
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

    pub fn register_component(&mut self, key: &str) -> Result<ComponentId, ModRegistrationError> {
        self.backend.register_component(key)
    }

    pub fn register_message(&mut self, key: &str) -> Result<MessageId, ModRegistrationError> {
        self.backend.register_message(key)
    }

    pub fn register_worldgen(
        &mut self,
        key: &str,
        factory: WorldGenFactory,
    ) -> Result<WorldGenId, ModRegistrationError> {
        self.backend.register_worldgen(key, factory)
    }

    pub fn register_character_controller(
        &mut self,
        key: &str,
        factory: CharacterControllerFactory,
    ) -> Result<CharacterControllerId, ModRegistrationError> {
        self.backend.register_character_controller(key, factory)
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

    pub fn on_server_tick(&mut self, hook: ServerTickHook) {
        self.backend.on_server_tick(hook);
    }

    pub fn on_client_tick(&mut self, hook: ClientTickHook) {
        self.backend.on_client_tick(hook);
    }
}

/// Numeric id for registered component keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u8);

/// Numeric id for registered component keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(pub u32);

/// Numeric id for registered message keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageId(pub u32);

/// Numeric id for registered worldgen providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldGenId(pub u32);

/// Numeric id for registered character controllers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CharacterControllerId(pub u32);

/// Numeric id for registered modnet channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelId(pub u32);

/// Error type for mod registration failures.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ModRegistrationError {
    #[error("duplicate {registry} key '{key}' registered by mod '{mod_id}'")]
    DuplicateKey {
        mod_id: String,
        registry: &'static str,
        key: String,
    },
    #[error("too many blocks registered by mod '{mod_id}' for key '{key}': limit is {limit}")]
    TooManyBlocks {
        mod_id: String,
        key: String,
        limit: u32,
    },
}

/// Error type for mod config decode failures.
#[derive(Debug, thiserror::Error)]
pub enum ModConfigError {
    #[error("failed to decode config for mod '{mod_id}'")]
    Deserialize {
        mod_id: String,
        #[source]
        source: toml::de::Error,
    },
}

/// Hook callback executed on server ticks.
pub type ServerTickHook = for<'a> fn(&mut ServerHookCtx<'a>);

/// Hook callback executed on client frame/tick updates.
pub type ClientTickHook = for<'a> fn(&mut ClientHookCtx<'a>);

/// Runtime-provided services exposed to SDK hooks.
pub trait Services {}

/// Empty services implementation used by runtimes that do not expose services yet.
#[derive(Debug, Default)]
pub struct NoServices;

impl Services for NoServices {}

/// Stable server hook context.
pub struct ServerHookCtx<'a> {
    pub tick: u64,
    pub dt: Duration,
    pub services: &'a mut dyn Services,
}

impl<'a> ServerHookCtx<'a> {
    #[must_use]
    pub fn new(tick: u64, dt: Duration, services: &'a mut dyn Services) -> Self {
        Self { tick, dt, services }
    }
}

/// Stable client hook context.
pub struct ClientHookCtx<'a> {
    pub tick: u64,
    pub dt: Duration,
    pub services: &'a mut dyn Services,
}

impl<'a> ClientHookCtx<'a> {
    #[must_use]
    pub fn new(tick: u64, dt: Duration, services: &'a mut dyn Services) -> Self {
        Self { tick, dt, services }
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
pub type WorldGenFactory = fn(WorldGenInit) -> Box<dyn WorldGenProvider>;

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

/// Raw network input command surface used by controller implementations.
#[derive(Debug, Clone, Copy, Default)]
pub struct RawInput {
    pub move_x: i8,
    pub move_z: i8,
    pub buttons: u16,
    pub yaw_q: i16,
    pub pitch_q: i16,
}

impl RawInput {
    #[inline]
    #[must_use]
    pub fn yaw_deg(&self) -> f32 {
        self.yaw_q as f32 / 100.0
    }

    #[inline]
    #[must_use]
    pub fn pitch_deg(&self) -> f32 {
        self.pitch_q as f32 / 100.0
    }
}

/// Canonical button bits for `RawInput::buttons`.
pub mod button_bits {
    pub const JUMP: u16 = 1;
    pub const SPRINT: u16 = 2;
    pub const CROUCH: u16 = 4;
}

/// Semantic movement intent derived from raw input.
#[derive(Debug, Clone, Copy, Default)]
pub struct CharacterIntent {
    pub move_x: f32,
    pub move_z: f32,
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub jump: bool,
    pub crouch: bool,
    pub sprint: bool,
}

/// Character shape used for collision queries.
#[derive(Debug, Clone, Copy)]
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
    fn intent_from_raw(&mut self, raw: &RawInput) -> CharacterIntent;
    fn step(
        &mut self,
        state: &mut CharacterState,
        intent: &CharacterIntent,
        physics: &mut dyn CharacterPhysics,
        dt: Duration,
    );
}

/// Character controller factory init parameters.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct CharacterControllerInit {}

/// Character controller factory.
pub type CharacterControllerFactory = fn(CharacterControllerInit) -> Box<dyn CharacterController>;

/// Channel reliability policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelReliability {
    Reliable,
    Unreliable,
}

/// Channel ordering policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelOrdering {
    Ordered,
    Unordered,
}

/// Channel traffic direction policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
