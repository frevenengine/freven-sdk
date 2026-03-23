use std::sync::Arc;

pub use freven_world_guest::{
    ClientKeyCode as GuestClientKeyCode, ClientMouseButton as GuestClientMouseButton,
};

/// Client control provider output for one input sample.
///
/// Notes:
/// - The engine owns input sequencing as part of the prediction/network timeline.
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

/// Init params for client control provider factories.
///
/// Reserved for future evolution.
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct ClientControlProviderInit {}

/// Mouse buttons for client input polling/consumption.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ClientMouseButton {
    Left,
    Right,
    Middle,
}

/// Keyboard keys for client input polling/consumption.
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

/// Contract for gameplay control providers owned by mods.
///
/// This is a pure mapping: device state -> raw input.
/// Providers may keep internal filters, but must not own network sequencing.
pub trait ClientControlProvider: Send + Sync {
    fn sample(&mut self, device: &mut dyn ClientControlDeviceState) -> ClientControlOutput;

    /// Optional hook to clear internal filters on hard resets.
    fn reset(&mut self) {}
}

/// Factory for client control providers.
pub type ClientControlProviderFactory =
    Arc<dyn Fn(ClientControlProviderInit) -> Box<dyn ClientControlProvider> + Send + Sync>;

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
