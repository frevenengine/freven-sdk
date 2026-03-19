use freven_mod_api::{HostExecutionKind, LogPayload, RuntimeSessionSide};

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
