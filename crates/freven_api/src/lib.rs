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
    fn register_channel(
        &mut self,
        key: &str,
        config: ChannelConfig,
    ) -> Result<ChannelId, ModRegistrationError>;
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

    pub fn register_channel(
        &mut self,
        key: &str,
        config: ChannelConfig,
    ) -> Result<ChannelId, ModRegistrationError> {
        self.backend.register_channel(key, config)
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
