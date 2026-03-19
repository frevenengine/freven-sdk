use std::sync::Arc;

use freven_block_sdk_types::BlockDescriptor;
use freven_mod_api::{
    CapabilityDeclaration, ChannelConfig, ChannelOrdering, ChannelReliability, ComponentCodec,
    MessageCodec, ModSide, Side,
};
use serde::de::DeserializeOwned;

use crate::{
    action::{ActionHandler, ActionKindId},
    character::{
        CharacterController, CharacterControllerFactory, CharacterControllerInit,
        ClientControlProvider, ClientControlProviderFactory, ClientControlProviderInit,
    },
    lifecycle::{
        ClientMessagesHook, ServerMessagesHook, StartClientHook, StartServerHook, TickClientHook,
        TickServerHook,
    },
    worldgen::{WorldGenFactory, WorldGenInit, WorldGenProvider},
};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredBlock {
    key: String,
}

impl RegisteredBlock {
    #[must_use]
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }

    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }
}

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

/// Numeric id for registered client control providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientControlProviderId(pub u32);

/// Message type registration config.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageConfig {
    pub codec: MessageCodec,
}

impl Default for MessageConfig {
    fn default() -> Self {
        Self {
            codec: MessageCodec::RawBytes,
        }
    }
}

/// Backend implemented by runtime for registration operations.
pub trait ModContextBackend {
    fn register_block(
        &mut self,
        key: &str,
        def: BlockDescriptor,
    ) -> Result<RegisteredBlock, ModRegistrationError>;
    fn register_component(
        &mut self,
        key: &str,
        codec: ComponentCodec,
    ) -> Result<ComponentId, ModRegistrationError>;
    fn register_message(
        &mut self,
        key: &str,
        config: MessageConfig,
    ) -> Result<MessageId, ModRegistrationError>;
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
    fn register_client_control_provider(
        &mut self,
        key: &str,
        factory: ClientControlProviderFactory,
    ) -> Result<ClientControlProviderId, ModRegistrationError>;
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
    fn register_action_kind(&mut self, key: &str) -> Result<ActionKindId, ModRegistrationError>;
    fn declare_capability(
        &mut self,
        capability: CapabilityDeclaration,
    ) -> Result<(), ModRegistrationError>;
    fn on_start_client(&mut self, hook: StartClientHook);
    fn on_start_server(&mut self, hook: StartServerHook);
    fn on_tick_client(&mut self, hook: TickClientHook);
    fn on_tick_server(&mut self, hook: TickServerHook);
    fn on_client_messages(&mut self, hook: ClientMessagesHook);
    fn on_server_messages(&mut self, hook: ServerMessagesHook);
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
        def: BlockDescriptor,
    ) -> Result<RegisteredBlock, ModRegistrationError> {
        self.backend.register_block(key, def)
    }

    pub fn register_component(
        &mut self,
        key: &str,
        codec: ComponentCodec,
    ) -> Result<ComponentId, ModRegistrationError> {
        self.backend.register_component(key, codec)
    }

    pub fn register_message(&mut self, key: &str) -> Result<MessageId, ModRegistrationError> {
        self.backend.register_message(key, MessageConfig::default())
    }

    pub fn register_message_type(
        &mut self,
        key: &str,
        config: MessageConfig,
    ) -> Result<MessageId, ModRegistrationError> {
        self.backend.register_message(key, config)
    }

    pub fn register_worldgen(
        &mut self,
        key: &str,
        factory: impl Fn(WorldGenInit) -> Box<dyn WorldGenProvider> + Send + Sync + 'static,
    ) -> Result<WorldGenId, ModRegistrationError> {
        self.backend.register_worldgen(key, Arc::new(factory))
    }

    pub fn register_character_controller(
        &mut self,
        key: &str,
        factory: impl Fn(CharacterControllerInit) -> Box<dyn CharacterController>
        + Send
        + Sync
        + 'static,
    ) -> Result<CharacterControllerId, ModRegistrationError> {
        self.backend
            .register_character_controller(key, Arc::new(factory))
    }

    pub fn register_client_control_provider(
        &mut self,
        key: &str,
        factory: impl Fn(ClientControlProviderInit) -> Box<dyn ClientControlProvider>
        + Send
        + Sync
        + 'static,
    ) -> Result<ClientControlProviderId, ModRegistrationError> {
        self.backend
            .register_client_control_provider(key, Arc::new(factory))
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

    pub fn register_action_kind(
        &mut self,
        key: &str,
    ) -> Result<ActionKindId, ModRegistrationError> {
        self.backend.register_action_kind(key)
    }

    pub fn declare_capability(&mut self, key: &str) -> Result<(), ModRegistrationError> {
        self.declare_capability_declaration(CapabilityDeclaration {
            key: key.to_string(),
        })
    }

    pub fn declare_capability_declaration(
        &mut self,
        capability: CapabilityDeclaration,
    ) -> Result<(), ModRegistrationError> {
        self.backend.declare_capability(capability)
    }

    pub fn on_start_client(&mut self, hook: StartClientHook) {
        self.backend.on_start_client(hook);
    }

    pub fn on_start_server(&mut self, hook: StartServerHook) {
        self.backend.on_start_server(hook);
    }

    pub fn on_tick_client(&mut self, hook: TickClientHook) {
        self.backend.on_tick_client(hook);
    }

    pub fn on_tick_server(&mut self, hook: TickServerHook) {
        self.backend.on_tick_server(hook);
    }

    pub fn on_client_messages(&mut self, hook: ClientMessagesHook) {
        self.backend.on_client_messages(hook);
    }

    pub fn on_server_messages(&mut self, hook: ServerMessagesHook) {
        self.backend.on_server_messages(hook);
    }
}

/// Error type for mod registration failures.
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum ModRegistrationError {
    #[error("duplicate {registry} key '{key}' registered by mod '{mod_id}'")]
    DuplicateKey {
        mod_id: String,
        registry: &'static str,
        key: String,
    },
    #[error("empty {registry} key registered by mod '{mod_id}'")]
    EmptyKey {
        mod_id: String,
        registry: &'static str,
    },
    #[error(
        "runtime registry '{registry}' is exhausted while registering key '{key}' for mod '{mod_id}'"
    )]
    RegistryExhausted {
        mod_id: String,
        registry: &'static str,
        key: String,
    },
    #[error(
        "unsupported channel QoS for mod '{mod_id}' key '{key}': reliability={reliability:?}, ordering={ordering:?}; v1 supports only (Reliable, Ordered) and (Unreliable, Unordered)"
    )]
    UnsupportedChannelQos {
        mod_id: String,
        key: String,
        reliability: ChannelReliability,
        ordering: ChannelOrdering,
    },
    #[error(
        "mod '{mod_id}' declared capability '{key}' but it is not present in the resolved capability table"
    )]
    UndeclaredCapability { mod_id: String, key: String },
    #[error("invalid {kind} declaration for mod '{mod_id}': {reason}")]
    InvalidDeclaration {
        mod_id: String,
        kind: &'static str,
        reason: String,
    },
}

pub fn validate_capability_declaration(
    mod_id: &str,
    capability: &CapabilityDeclaration,
    allowed: Option<&toml::Table>,
) -> Result<(), ModRegistrationError> {
    if capability.key.trim().is_empty() {
        return Err(ModRegistrationError::EmptyKey {
            mod_id: mod_id.to_string(),
            registry: "capability",
        });
    }
    if let Some(allowed) = allowed
        && !allowed.contains_key(&capability.key)
    {
        return Err(ModRegistrationError::UndeclaredCapability {
            mod_id: mod_id.to_string(),
            key: capability.key.clone(),
        });
    }
    Ok(())
}

/// Error type for mod config decode failures.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ModConfigError {
    #[error("failed to decode config for mod '{mod_id}'")]
    Deserialize {
        mod_id: String,
        #[source]
        source: toml::de::Error,
    },
}
