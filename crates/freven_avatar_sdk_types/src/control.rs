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
///
/// This avatar-owned surface stays semantically synchronized with the
/// runtime-loaded guest transport equivalent in `freven_world_guest`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ClientMouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

/// Physical keyboard keys for client input polling/consumption.
///
/// Names intentionally follow W3C `KeyboardEvent.code` / winit-style physical
/// key naming where practical. This surface is for gameplay bindings, not text
/// input.
///
/// Use `Digit1`..`Digit9` for hotbars and number-row gameplay bindings.
/// Use `KeyA`..`KeyZ` for physical letter-key positions even when printed
/// legends differ by keyboard layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ClientKeyCode {
    Backquote,
    Backslash,
    BracketLeft,
    BracketRight,
    Comma,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Equal,
    IntlBackslash,
    IntlRo,
    IntlYen,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Minus,
    Period,
    Quote,
    Semicolon,
    Slash,
    AltLeft,
    AltRight,
    Backspace,
    CapsLock,
    ContextMenu,
    ControlLeft,
    ControlRight,
    Enter,
    SuperLeft,
    SuperRight,
    ShiftLeft,
    ShiftRight,
    Space,
    Tab,
    Convert,
    KanaMode,
    Lang1,
    Lang2,
    Lang3,
    Lang4,
    Lang5,
    NonConvert,
    Delete,
    End,
    Help,
    Home,
    Insert,
    PageDown,
    PageUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    NumLock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadBackspace,
    NumpadClear,
    NumpadClearEntry,
    NumpadComma,
    NumpadDecimal,
    NumpadDivide,
    NumpadEnter,
    NumpadEqual,
    NumpadHash,
    NumpadMemoryAdd,
    NumpadMemoryClear,
    NumpadMemoryRecall,
    NumpadMemoryStore,
    NumpadMemorySubtract,
    NumpadMultiply,
    NumpadParenLeft,
    NumpadParenRight,
    NumpadStar,
    NumpadSubtract,
    Escape,
    Fn,
    FnLock,
    PrintScreen,
    ScrollLock,
    Pause,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,
    BrowserBack,
    BrowserFavorites,
    BrowserForward,
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,
    Eject,
    LaunchApp1,
    LaunchApp2,
    LaunchMail,
    MediaPlayPause,
    MediaSelect,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    Power,
    Sleep,
    AudioVolumeDown,
    AudioVolumeMute,
    AudioVolumeUp,
    WakeUp,
    Again,
    Copy,
    Cut,
    Find,
    Open,
    Paste,
    Props,
    Select,
    Undo,
    /// Compatibility aggregate. New bindings should prefer `ShiftLeft` / `ShiftRight`.
    Shift,
    /// Compatibility aggregate. New bindings should prefer `ControlLeft` / `ControlRight`.
    Ctrl,
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

#[cfg(test)]
mod tests {
    use super::{ClientKeyCode, ClientMouseButton, GuestClientKeyCode, GuestClientMouseButton};

    macro_rules! same_mouse_buttons {
        ($value:expr, $source:ident, $target:ident) => {
            match $value {
                $source::Left => $target::Left,
                $source::Right => $target::Right,
                $source::Middle => $target::Middle,
                $source::Back => $target::Back,
                $source::Forward => $target::Forward,
                $source::Other(button) => $target::Other(button),
            }
        };
    }

    macro_rules! same_key_codes {
        ($value:expr, $source:ident, $target:ident) => {
            match $value {
                $source::Backquote => $target::Backquote,
                $source::Backslash => $target::Backslash,
                $source::BracketLeft => $target::BracketLeft,
                $source::BracketRight => $target::BracketRight,
                $source::Comma => $target::Comma,
                $source::Digit0 => $target::Digit0,
                $source::Digit1 => $target::Digit1,
                $source::Digit2 => $target::Digit2,
                $source::Digit3 => $target::Digit3,
                $source::Digit4 => $target::Digit4,
                $source::Digit5 => $target::Digit5,
                $source::Digit6 => $target::Digit6,
                $source::Digit7 => $target::Digit7,
                $source::Digit8 => $target::Digit8,
                $source::Digit9 => $target::Digit9,
                $source::Equal => $target::Equal,
                $source::IntlBackslash => $target::IntlBackslash,
                $source::IntlRo => $target::IntlRo,
                $source::IntlYen => $target::IntlYen,
                $source::KeyA => $target::KeyA,
                $source::KeyB => $target::KeyB,
                $source::KeyC => $target::KeyC,
                $source::KeyD => $target::KeyD,
                $source::KeyE => $target::KeyE,
                $source::KeyF => $target::KeyF,
                $source::KeyG => $target::KeyG,
                $source::KeyH => $target::KeyH,
                $source::KeyI => $target::KeyI,
                $source::KeyJ => $target::KeyJ,
                $source::KeyK => $target::KeyK,
                $source::KeyL => $target::KeyL,
                $source::KeyM => $target::KeyM,
                $source::KeyN => $target::KeyN,
                $source::KeyO => $target::KeyO,
                $source::KeyP => $target::KeyP,
                $source::KeyQ => $target::KeyQ,
                $source::KeyR => $target::KeyR,
                $source::KeyS => $target::KeyS,
                $source::KeyT => $target::KeyT,
                $source::KeyU => $target::KeyU,
                $source::KeyV => $target::KeyV,
                $source::KeyW => $target::KeyW,
                $source::KeyX => $target::KeyX,
                $source::KeyY => $target::KeyY,
                $source::KeyZ => $target::KeyZ,
                $source::Minus => $target::Minus,
                $source::Period => $target::Period,
                $source::Quote => $target::Quote,
                $source::Semicolon => $target::Semicolon,
                $source::Slash => $target::Slash,
                $source::AltLeft => $target::AltLeft,
                $source::AltRight => $target::AltRight,
                $source::Backspace => $target::Backspace,
                $source::CapsLock => $target::CapsLock,
                $source::ContextMenu => $target::ContextMenu,
                $source::ControlLeft => $target::ControlLeft,
                $source::ControlRight => $target::ControlRight,
                $source::Enter => $target::Enter,
                $source::SuperLeft => $target::SuperLeft,
                $source::SuperRight => $target::SuperRight,
                $source::ShiftLeft => $target::ShiftLeft,
                $source::ShiftRight => $target::ShiftRight,
                $source::Space => $target::Space,
                $source::Tab => $target::Tab,
                $source::Convert => $target::Convert,
                $source::KanaMode => $target::KanaMode,
                $source::Lang1 => $target::Lang1,
                $source::Lang2 => $target::Lang2,
                $source::Lang3 => $target::Lang3,
                $source::Lang4 => $target::Lang4,
                $source::Lang5 => $target::Lang5,
                $source::NonConvert => $target::NonConvert,
                $source::Delete => $target::Delete,
                $source::End => $target::End,
                $source::Help => $target::Help,
                $source::Home => $target::Home,
                $source::Insert => $target::Insert,
                $source::PageDown => $target::PageDown,
                $source::PageUp => $target::PageUp,
                $source::ArrowDown => $target::ArrowDown,
                $source::ArrowLeft => $target::ArrowLeft,
                $source::ArrowRight => $target::ArrowRight,
                $source::ArrowUp => $target::ArrowUp,
                $source::NumLock => $target::NumLock,
                $source::Numpad0 => $target::Numpad0,
                $source::Numpad1 => $target::Numpad1,
                $source::Numpad2 => $target::Numpad2,
                $source::Numpad3 => $target::Numpad3,
                $source::Numpad4 => $target::Numpad4,
                $source::Numpad5 => $target::Numpad5,
                $source::Numpad6 => $target::Numpad6,
                $source::Numpad7 => $target::Numpad7,
                $source::Numpad8 => $target::Numpad8,
                $source::Numpad9 => $target::Numpad9,
                $source::NumpadAdd => $target::NumpadAdd,
                $source::NumpadBackspace => $target::NumpadBackspace,
                $source::NumpadClear => $target::NumpadClear,
                $source::NumpadClearEntry => $target::NumpadClearEntry,
                $source::NumpadComma => $target::NumpadComma,
                $source::NumpadDecimal => $target::NumpadDecimal,
                $source::NumpadDivide => $target::NumpadDivide,
                $source::NumpadEnter => $target::NumpadEnter,
                $source::NumpadEqual => $target::NumpadEqual,
                $source::NumpadHash => $target::NumpadHash,
                $source::NumpadMemoryAdd => $target::NumpadMemoryAdd,
                $source::NumpadMemoryClear => $target::NumpadMemoryClear,
                $source::NumpadMemoryRecall => $target::NumpadMemoryRecall,
                $source::NumpadMemoryStore => $target::NumpadMemoryStore,
                $source::NumpadMemorySubtract => $target::NumpadMemorySubtract,
                $source::NumpadMultiply => $target::NumpadMultiply,
                $source::NumpadParenLeft => $target::NumpadParenLeft,
                $source::NumpadParenRight => $target::NumpadParenRight,
                $source::NumpadStar => $target::NumpadStar,
                $source::NumpadSubtract => $target::NumpadSubtract,
                $source::Escape => $target::Escape,
                $source::Fn => $target::Fn,
                $source::FnLock => $target::FnLock,
                $source::PrintScreen => $target::PrintScreen,
                $source::ScrollLock => $target::ScrollLock,
                $source::Pause => $target::Pause,
                $source::F1 => $target::F1,
                $source::F2 => $target::F2,
                $source::F3 => $target::F3,
                $source::F4 => $target::F4,
                $source::F5 => $target::F5,
                $source::F6 => $target::F6,
                $source::F7 => $target::F7,
                $source::F8 => $target::F8,
                $source::F9 => $target::F9,
                $source::F10 => $target::F10,
                $source::F11 => $target::F11,
                $source::F12 => $target::F12,
                $source::F13 => $target::F13,
                $source::F14 => $target::F14,
                $source::F15 => $target::F15,
                $source::F16 => $target::F16,
                $source::F17 => $target::F17,
                $source::F18 => $target::F18,
                $source::F19 => $target::F19,
                $source::F20 => $target::F20,
                $source::F21 => $target::F21,
                $source::F22 => $target::F22,
                $source::F23 => $target::F23,
                $source::F24 => $target::F24,
                $source::F25 => $target::F25,
                $source::F26 => $target::F26,
                $source::F27 => $target::F27,
                $source::F28 => $target::F28,
                $source::F29 => $target::F29,
                $source::F30 => $target::F30,
                $source::F31 => $target::F31,
                $source::F32 => $target::F32,
                $source::F33 => $target::F33,
                $source::F34 => $target::F34,
                $source::F35 => $target::F35,
                $source::BrowserBack => $target::BrowserBack,
                $source::BrowserFavorites => $target::BrowserFavorites,
                $source::BrowserForward => $target::BrowserForward,
                $source::BrowserHome => $target::BrowserHome,
                $source::BrowserRefresh => $target::BrowserRefresh,
                $source::BrowserSearch => $target::BrowserSearch,
                $source::BrowserStop => $target::BrowserStop,
                $source::Eject => $target::Eject,
                $source::LaunchApp1 => $target::LaunchApp1,
                $source::LaunchApp2 => $target::LaunchApp2,
                $source::LaunchMail => $target::LaunchMail,
                $source::MediaPlayPause => $target::MediaPlayPause,
                $source::MediaSelect => $target::MediaSelect,
                $source::MediaStop => $target::MediaStop,
                $source::MediaTrackNext => $target::MediaTrackNext,
                $source::MediaTrackPrevious => $target::MediaTrackPrevious,
                $source::Power => $target::Power,
                $source::Sleep => $target::Sleep,
                $source::AudioVolumeDown => $target::AudioVolumeDown,
                $source::AudioVolumeMute => $target::AudioVolumeMute,
                $source::AudioVolumeUp => $target::AudioVolumeUp,
                $source::WakeUp => $target::WakeUp,
                $source::Again => $target::Again,
                $source::Copy => $target::Copy,
                $source::Cut => $target::Cut,
                $source::Find => $target::Find,
                $source::Open => $target::Open,
                $source::Paste => $target::Paste,
                $source::Props => $target::Props,
                $source::Select => $target::Select,
                $source::Undo => $target::Undo,
                $source::Shift => $target::Shift,
                $source::Ctrl => $target::Ctrl,
            }
        };
    }

    fn avatar_mouse_to_guest(button: ClientMouseButton) -> GuestClientMouseButton {
        same_mouse_buttons!(button, ClientMouseButton, GuestClientMouseButton)
    }

    fn guest_mouse_to_avatar(button: GuestClientMouseButton) -> ClientMouseButton {
        same_mouse_buttons!(button, GuestClientMouseButton, ClientMouseButton)
    }

    fn avatar_key_to_guest(key: ClientKeyCode) -> GuestClientKeyCode {
        same_key_codes!(key, ClientKeyCode, GuestClientKeyCode)
    }

    fn guest_key_to_avatar(key: GuestClientKeyCode) -> ClientKeyCode {
        same_key_codes!(key, GuestClientKeyCode, ClientKeyCode)
    }

    #[test]
    fn avatar_and_guest_mouse_button_surfaces_stay_synchronized() {
        let buttons = [
            ClientMouseButton::Left,
            ClientMouseButton::Right,
            ClientMouseButton::Middle,
            ClientMouseButton::Back,
            ClientMouseButton::Forward,
            ClientMouseButton::Other(8),
        ];

        for button in buttons {
            assert_eq!(guest_mouse_to_avatar(avatar_mouse_to_guest(button)), button);
        }
    }

    #[test]
    fn avatar_and_guest_key_code_surfaces_stay_synchronized_for_representative_variants() {
        let keys = [
            ClientKeyCode::Digit1,
            ClientKeyCode::KeyY,
            ClientKeyCode::KeyZ,
            ClientKeyCode::AltLeft,
            ClientKeyCode::ControlRight,
            ClientKeyCode::ShiftRight,
            ClientKeyCode::Space,
            ClientKeyCode::Tab,
            ClientKeyCode::ArrowLeft,
            ClientKeyCode::Numpad1,
            ClientKeyCode::Escape,
            ClientKeyCode::F12,
            ClientKeyCode::Shift,
            ClientKeyCode::Ctrl,
        ];

        for key in keys {
            assert_eq!(guest_key_to_avatar(avatar_key_to_guest(key)), key);
        }
    }
}
