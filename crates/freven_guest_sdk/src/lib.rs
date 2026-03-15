//! Neutral guest authoring helpers.

pub use freven_guest::{
    CapabilityDeclaration, ChannelBudget, ChannelConfig, ChannelDeclaration, ChannelDirection,
    ChannelOrdering, ChannelReliability, ComponentCodec, ComponentDeclaration, GuestCallbacks,
    GuestDescription, GuestRegistration, LifecycleHooks, LogLevel, LogPayload, MessageCodec,
    MessageDeclaration, MessageHooks, NegotiationRequest, NegotiationResponse, RuntimeSessionInfo,
    RuntimeSessionSide,
};
pub use freven_mod_api::{ObservabilityBridge, emit_log};

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {{
        $crate::emit_log($crate::LogLevel::Debug, format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {{
        $crate::emit_log($crate::LogLevel::Info, format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {{
        $crate::emit_log($crate::LogLevel::Warn, format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {{
        $crate::emit_log($crate::LogLevel::Error, format!($($arg)*));
    }};
}
