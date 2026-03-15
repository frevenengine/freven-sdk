//! Neutral SDK contracts for platform-facing Freven authoring surfaces.

use std::{cell::RefCell, ffi::c_void};

pub use freven_guest::{
    CapabilityDeclaration, ChannelBudget, ChannelConfig, ChannelDeclaration, ChannelDirection,
    ChannelOrdering, ChannelReliability, ComponentCodec, ComponentDeclaration, GuestCallbacks,
    GuestDescription, GuestRegistration, LifecycleHooks, LogLevel, LogPayload, MessageCodec,
    MessageDeclaration, MessageHooks, NegotiationRequest, NegotiationResponse, RuntimeSessionInfo,
    RuntimeSessionSide,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Client,
    Server,
}

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
            (Self::Client, Side::Client)
                | (Self::Server, Side::Server)
                | (Self::Both, Side::Client)
                | (Self::Both, Side::Server)
        )
    }
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

pub mod __private {
    use super::ObservabilityBridge;

    pub fn with_observability_bridge<T>(bridge: ObservabilityBridge, f: impl FnOnce() -> T) -> T {
        super::OBSERVABILITY_BRIDGE.with(|slot| {
            let prev = slot.replace(bridge);
            let out = f();
            slot.replace(prev);
            out
        })
    }
}
