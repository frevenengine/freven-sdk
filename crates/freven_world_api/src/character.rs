use crate::ClientControlDeviceState;
use std::{sync::Arc, time::Duration};

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
