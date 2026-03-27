//! High-level guest authoring helpers built on top of `freven_guest`.

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::cell::RefCell;
use core::ffi::c_void;
use std::thread::LocalKey;

pub use freven_block_guest::{
    BlockClientQueryRequest, BlockClientQueryResponse, BlockMutation, BlockMutationBatch,
    BlockQueryRequest, BlockQueryResponse, BlockServiceRequest, BlockServiceResponse,
};
pub use freven_block_sdk_types::{
    BlockCollision, BlockDescriptor, BlockMaterial, BlockRuntimeId, BlockVisibility, RenderLayer,
};
use freven_guest::{
    CapabilityDeclaration, ChannelConfig, ChannelDeclaration, ComponentCodec, ComponentDeclaration,
    LifecycleHooks, LogLevel, LogPayload, MessageCodec, MessageDeclaration, MessageHooks,
    NegotiationRequest, RuntimeSessionInfo, RuntimeSessionSide,
};
pub use freven_volumetric_api::{
    InitialWorldSpawnHint, WorldGenBootstrapOutput, WorldGenInit, WorldGenOutput, WorldGenRequest,
    WorldTerrainWrite,
};
pub use freven_world_guest::{
    ActionDeclaration, ActionInput, ActionOutcome, ActionResult, AvatarGuestRegistration,
    AvatarProviderHooks, BlockDeclaration, CharacterConfig, CharacterControllerDeclaration,
    CharacterControllerInitInput, CharacterControllerInitResult, CharacterControllerInput,
    CharacterControllerStepInput, CharacterControllerStepResult, CharacterShape, CharacterState,
    ClientControlOutput, ClientControlProviderDeclaration, ClientControlSampleInput,
    ClientControlSampleResult, ClientInboundMessage, ClientKeyCode, ClientMessageInput,
    ClientMessageResult, ClientMouseButton, ClientOutboundMessage, ClientOutboundMessageScope,
    ClientPlayerView, ClientVisibilityRequest, ClientVisibilityResponse, GuestCallbacks,
    GuestDescription, GuestRegistration, InputTimeline, KinematicMoveConfig, KinematicMoveResult,
    LifecycleResult, MessageScope, ModConfigDocument, ModConfigFormat, NegotiationResponse,
    ProviderHooks, RuntimeCharacterPhysicsRequest, RuntimeClientControlRequest, RuntimeLevelRef,
    RuntimeMessageOutput, RuntimeObservabilityRequest, RuntimeOutput, ServerInboundMessage,
    ServerMessageInput, ServerMessageResult, ServerOutboundMessage, StartInput, SweepHit,
    TickInput, WorldGenCallInput, WorldGenCallResult, WorldGenDeclaration, WorldGuestRegistration,
    WorldProviderHooks, WorldQueryRequest, WorldQueryResponse, WorldServiceRequest,
    WorldServiceResponse, WorldSessionRequest, WorldSessionResponse,
};
use serde::de::DeserializeOwned;

type StartHandler = fn(StartContext<'_>) -> LifecycleResult;
type TickHandler = fn(TickContext<'_>) -> LifecycleResult;
type ActionHandler = fn(ActionContext<'_>) -> ActionResult;
type ClientMessageHandler = fn(ClientMessageContext<'_>) -> ClientMessageResponse;
type ServerMessageHandler = fn(ServerMessageContext<'_>) -> ServerMessageResponse;
type WorldGenHandler = fn(WorldGenContext<'_>) -> WorldGenCallResult;
type CharacterControllerInitHandler =
    fn(CharacterControllerInitContext<'_>) -> CharacterControllerInitResult;
type CharacterControllerStepHandler =
    fn(CharacterControllerStepContext<'_>) -> CharacterControllerStepResult;
type ClientControlProviderHandler =
    fn(ClientControlProviderContext<'_>) -> ClientControlSampleResult;
type StatefulSessionFactory<S> = fn(StartContext<'_>) -> S;
type StatefulStartHandler<S> = fn(&mut S, StartContext<'_>) -> LifecycleResult;
type StatefulTickHandler<S> = fn(&mut S, TickContext<'_>) -> LifecycleResult;
type StatefulActionHandler<S> = fn(&mut S, ActionContext<'_>) -> ActionResult;
type StatefulClientMessageHandler<S> =
    fn(&mut S, ClientMessageContext<'_>) -> ClientMessageResponse;
type StatefulServerMessageHandler<S> =
    fn(&mut S, ServerMessageContext<'_>) -> ServerMessageResponse;

pub type NativeRuntimeServiceCall = unsafe extern "C" fn(
    ctx: *mut c_void,
    req_ptr: *const u8,
    req_len: usize,
    resp_ptr: *mut u8,
    resp_cap: usize,
) -> usize;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeGuestInput {
    pub ptr: *const u8,
    pub len: usize,
}

impl NativeGuestInput {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            ptr: core::ptr::null(),
            len: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeGuestBuffer {
    pub ptr: *mut u8,
    pub len: usize,
}

impl NativeGuestBuffer {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            ptr: core::ptr::null_mut(),
            len: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NativeRuntimeBridge {
    pub ctx: *mut c_void,
    pub call: Option<NativeRuntimeServiceCall>,
}

impl NativeRuntimeBridge {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            ctx: core::ptr::null_mut(),
            call: None,
        }
    }
}

thread_local! {
    static NATIVE_RUNTIME_BRIDGE: RefCell<NativeRuntimeBridge> =
        const { RefCell::new(NativeRuntimeBridge::empty()) };
}

pub struct GuestModule {
    guest_id: &'static str,
    blocks: Vec<BlockDeclaration>,
    components: Vec<ComponentDeclaration>,
    messages: Vec<MessageDeclaration>,
    worldgen: Vec<WorldGenDeclaration>,
    character_controllers: Vec<CharacterControllerDeclaration>,
    client_control_providers: Vec<ClientControlProviderDeclaration>,
    worldgen_handlers: Vec<GuestWorldGen>,
    character_controller_handlers: Vec<GuestCharacterController>,
    client_control_provider_handlers: Vec<GuestClientControlProvider>,
    channels: Vec<ChannelDeclaration>,
    actions: Vec<GuestAction>,
    capabilities: Vec<CapabilityDeclaration>,
    on_start_client: Option<StartHandler>,
    on_start_server: Option<StartHandler>,
    on_tick_client: Option<TickHandler>,
    on_tick_server: Option<TickHandler>,
    on_client_messages: Option<ClientMessageHandler>,
    on_server_messages: Option<ServerMessageHandler>,
}

impl GuestModule {
    #[must_use]
    pub fn new(guest_id: &'static str) -> Self {
        assert!(
            !guest_id.trim().is_empty(),
            "freven_guest_sdk guest_id must not be empty"
        );
        Self {
            guest_id,
            blocks: Vec::new(),
            components: Vec::new(),
            messages: Vec::new(),
            worldgen: Vec::new(),
            character_controllers: Vec::new(),
            client_control_providers: Vec::new(),
            worldgen_handlers: Vec::new(),
            character_controller_handlers: Vec::new(),
            client_control_provider_handlers: Vec::new(),
            channels: Vec::new(),
            actions: Vec::new(),
            capabilities: Vec::new(),
            on_start_client: None,
            on_start_server: None,
            on_tick_client: None,
            on_tick_server: None,
            on_client_messages: None,
            on_server_messages: None,
        }
    }

    #[must_use]
    pub fn register_block(mut self, key: &'static str, def: BlockDescriptor) -> Self {
        assert_unique_key(
            "block",
            key,
            self.blocks.iter().map(|entry| entry.key.as_str()),
        );
        self.blocks.push(BlockDeclaration {
            key: key.to_string(),
            def,
        });
        self
    }

    #[must_use]
    pub fn register_component(mut self, key: &'static str, codec: ComponentCodec) -> Self {
        assert_unique_key(
            "component",
            key,
            self.components.iter().map(|entry| entry.key.as_str()),
        );
        self.components.push(ComponentDeclaration {
            key: key.to_string(),
            codec,
        });
        self
    }

    #[must_use]
    pub fn register_message(mut self, key: &'static str, codec: MessageCodec) -> Self {
        assert_unique_key(
            "message",
            key,
            self.messages.iter().map(|entry| entry.key.as_str()),
        );
        self.messages.push(MessageDeclaration {
            key: key.to_string(),
            codec,
        });
        self
    }

    #[must_use]
    pub fn register_worldgen(mut self, key: &'static str) -> Self {
        assert_unique_key(
            "worldgen",
            key,
            self.worldgen.iter().map(|entry| entry.key.as_str()),
        );
        self.worldgen.push(WorldGenDeclaration {
            key: key.to_string(),
        });
        self
    }

    #[must_use]
    pub fn register_worldgen_handler(
        mut self,
        key: &'static str,
        handler: WorldGenHandler,
    ) -> Self {
        self = self.register_worldgen(key);
        self.worldgen_handlers.push(GuestWorldGen { key, handler });
        self
    }

    #[must_use]
    pub fn register_character_controller(mut self, key: &'static str) -> Self {
        assert_unique_key(
            "character_controller",
            key,
            self.character_controllers
                .iter()
                .map(|entry| entry.key.as_str()),
        );
        self.character_controllers
            .push(CharacterControllerDeclaration {
                key: key.to_string(),
            });
        self
    }

    #[must_use]
    pub fn register_character_controller_handler(
        mut self,
        key: &'static str,
        init: CharacterControllerInitHandler,
        step: CharacterControllerStepHandler,
    ) -> Self {
        self = self.register_character_controller(key);
        self.character_controller_handlers
            .push(GuestCharacterController { key, init, step });
        self
    }

    #[must_use]
    pub fn register_client_control_provider(mut self, key: &'static str) -> Self {
        assert_unique_key(
            "client_control_provider",
            key,
            self.client_control_providers
                .iter()
                .map(|entry| entry.key.as_str()),
        );
        self.client_control_providers
            .push(ClientControlProviderDeclaration {
                key: key.to_string(),
            });
        self
    }

    #[must_use]
    pub fn register_client_control_provider_handler(
        mut self,
        key: &'static str,
        handler: ClientControlProviderHandler,
    ) -> Self {
        self = self.register_client_control_provider(key);
        self.client_control_provider_handlers
            .push(GuestClientControlProvider { key, handler });
        self
    }

    #[must_use]
    pub fn register_channel(mut self, key: &'static str, config: ChannelConfig) -> Self {
        assert_unique_key(
            "channel",
            key,
            self.channels.iter().map(|entry| entry.key.as_str()),
        );
        self.channels.push(ChannelDeclaration {
            key: key.to_string(),
            config,
        });
        self
    }

    #[must_use]
    pub fn declare_capability(mut self, key: &'static str) -> Self {
        assert_unique_key(
            "capability",
            key,
            self.capabilities.iter().map(|entry| entry.key.as_str()),
        );
        self.capabilities.push(CapabilityDeclaration {
            key: key.to_string(),
        });
        self
    }

    #[must_use]
    pub fn on_start_client(mut self, handler: StartHandler) -> Self {
        self.on_start_client = Some(handler);
        self
    }

    #[must_use]
    pub fn on_start_server(mut self, handler: StartHandler) -> Self {
        self.on_start_server = Some(handler);
        self
    }

    #[must_use]
    pub fn on_tick_client(mut self, handler: TickHandler) -> Self {
        self.on_tick_client = Some(handler);
        self
    }

    #[must_use]
    pub fn on_tick_server(mut self, handler: TickHandler) -> Self {
        self.on_tick_server = Some(handler);
        self
    }

    #[must_use]
    pub fn on_client_messages(mut self, handler: ClientMessageHandler) -> Self {
        self.on_client_messages = Some(handler);
        self
    }

    #[must_use]
    pub fn on_server_messages(mut self, handler: ServerMessageHandler) -> Self {
        self.on_server_messages = Some(handler);
        self
    }

    #[must_use]
    pub fn action(mut self, key: &'static str, binding_id: u32, handler: ActionHandler) -> Self {
        assert_unique_key("action", key, self.actions.iter().map(|entry| entry.key));
        assert!(
            self.actions
                .iter()
                .all(|action| action.binding_id != binding_id),
            "freven_guest_sdk binding id {binding_id} was registered more than once"
        );
        self.actions.push(GuestAction {
            key,
            binding_id,
            handler,
        });
        self
    }

    #[must_use]
    pub fn guest_id(&self) -> &'static str {
        self.guest_id
    }

    #[must_use]
    pub fn lifecycle_hooks(&self) -> LifecycleHooks {
        LifecycleHooks {
            start_client: self.on_start_client.is_some(),
            start_server: self.on_start_server.is_some(),
            tick_client: self.on_tick_client.is_some(),
            tick_server: self.on_tick_server.is_some(),
        }
    }

    #[must_use]
    pub fn callbacks(&self) -> GuestCallbacks {
        GuestCallbacks {
            lifecycle: self.lifecycle_hooks(),
            action: !self.actions.is_empty(),
            messages: MessageHooks {
                client: self.on_client_messages.is_some(),
                server: self.on_server_messages.is_some(),
            },
            providers: ProviderHooks {
                world: WorldProviderHooks {
                    worldgen: !self.worldgen_handlers.is_empty(),
                },
                avatar: AvatarProviderHooks {
                    character_controller: !self.character_controller_handlers.is_empty(),
                    client_control_provider: !self.client_control_provider_handlers.is_empty(),
                },
            },
        }
    }

    #[must_use]
    pub fn description(&self) -> GuestDescription {
        GuestDescription {
            guest_id: self.guest_id.to_string(),
            registration: GuestRegistration {
                blocks: self.blocks.clone(),
                components: self.components.clone(),
                messages: self.messages.clone(),
                world: WorldGuestRegistration {
                    worldgen: self.worldgen.clone(),
                },
                avatar: AvatarGuestRegistration {
                    character_controllers: self.character_controllers.clone(),
                    client_control_providers: self.client_control_providers.clone(),
                },
                channels: self.channels.clone(),
                actions: self
                    .actions
                    .iter()
                    .map(|action| ActionDeclaration {
                        key: action.key.to_string(),
                        binding_id: action.binding_id,
                    })
                    .collect(),
                capabilities: self.capabilities.clone(),
            },
            callbacks: self.callbacks(),
        }
    }

    pub fn handle_start_client(&self, input: &StartInput) -> LifecycleResult {
        let Some(handler) = self.on_start_client else {
            return LifecycleResponse::default().finish();
        };
        handler(StartContext { input })
    }

    pub fn handle_start_server(&self, input: &StartInput) -> LifecycleResult {
        let Some(handler) = self.on_start_server else {
            return LifecycleResponse::default().finish();
        };
        handler(StartContext { input })
    }

    pub fn handle_tick_client(&self, input: &TickInput) -> LifecycleResult {
        let Some(handler) = self.on_tick_client else {
            return LifecycleResponse::default().finish();
        };
        handler(TickContext { input })
    }

    pub fn handle_tick_server(&self, input: &TickInput) -> LifecycleResult {
        let Some(handler) = self.on_tick_server else {
            return LifecycleResponse::default().finish();
        };
        handler(TickContext { input })
    }

    #[must_use]
    pub fn handle_client_messages(&self, input: ClientMessageInput) -> ClientMessageResult {
        let Some(handler) = self.on_client_messages else {
            return ClientMessageResponse::default().finish();
        };
        handler(ClientMessageContext { input: &input }).finish()
    }

    #[must_use]
    pub fn handle_action(&self, input: ActionInput<'_>) -> ActionResult {
        let Some(action) = self
            .actions
            .iter()
            .find(|action| action.binding_id == input.binding_id)
        else {
            return ActionResponse::rejected().finish();
        };

        (action.handler)(ActionContext { input })
    }

    #[must_use]
    pub fn handle_server_messages(&self, input: ServerMessageInput) -> ServerMessageResult {
        let Some(handler) = self.on_server_messages else {
            return ServerMessageResponse::default().finish();
        };
        handler(ServerMessageContext { input: &input }).finish()
    }

    #[must_use]
    pub fn handle_worldgen(&self, input: WorldGenCallInput) -> WorldGenCallResult {
        let Some(entry) = self
            .worldgen_handlers
            .iter()
            .find(|entry| entry.key == input.key)
        else {
            return WorldGenCallResult::default();
        };
        (entry.handler)(WorldGenContext { input: &input })
    }

    #[must_use]
    pub fn handle_character_controller_init(
        &self,
        input: CharacterControllerInitInput,
    ) -> CharacterControllerInitResult {
        let Some(entry) = self
            .character_controller_handlers
            .iter()
            .find(|entry| entry.key == input.key)
        else {
            panic!("freven_guest_sdk character controller init called for undeclared key");
        };
        (entry.init)(CharacterControllerInitContext { input: &input })
    }

    #[must_use]
    pub fn handle_character_controller_step(
        &self,
        input: CharacterControllerStepInput,
    ) -> CharacterControllerStepResult {
        let Some(entry) = self
            .character_controller_handlers
            .iter()
            .find(|entry| entry.key == input.key)
        else {
            return CharacterControllerStepResult { state: input.state };
        };
        (entry.step)(CharacterControllerStepContext { input: &input })
    }

    #[must_use]
    pub fn handle_client_control_provider(
        &self,
        input: ClientControlSampleInput,
    ) -> ClientControlSampleResult {
        let Some(entry) = self
            .client_control_provider_handlers
            .iter()
            .find(|entry| entry.key == input.key)
        else {
            return ClientControlSampleResult {
                output: ClientControlOutput {
                    input: Vec::new(),
                    view_yaw_deg_mdeg: 0,
                    view_pitch_deg_mdeg: 0,
                },
            };
        };
        (entry.handler)(ClientControlProviderContext { input: &input })
    }
}

#[doc(hidden)]
pub trait ExportedGuestModule {
    fn description(&self) -> GuestDescription;
    fn handle_start_client(&self, input: &StartInput) -> LifecycleResult;
    fn handle_start_server(&self, input: &StartInput) -> LifecycleResult;
    fn handle_tick_client(&self, input: &TickInput) -> LifecycleResult;
    fn handle_tick_server(&self, input: &TickInput) -> LifecycleResult;
    fn handle_client_messages(&self, input: ClientMessageInput) -> ClientMessageResult;
    fn handle_action(&self, input: ActionInput<'_>) -> ActionResult;
    fn handle_server_messages(&self, input: ServerMessageInput) -> ServerMessageResult;
    fn handle_worldgen(&self, input: WorldGenCallInput) -> WorldGenCallResult;
    fn handle_character_controller_init(
        &self,
        input: CharacterControllerInitInput,
    ) -> CharacterControllerInitResult;
    fn handle_character_controller_step(
        &self,
        input: CharacterControllerStepInput,
    ) -> CharacterControllerStepResult;
    fn handle_client_control_provider(
        &self,
        input: ClientControlSampleInput,
    ) -> ClientControlSampleResult;
}

impl ExportedGuestModule for GuestModule {
    fn description(&self) -> GuestDescription {
        GuestModule::description(self)
    }

    fn handle_start_client(&self, input: &StartInput) -> LifecycleResult {
        GuestModule::handle_start_client(self, input)
    }

    fn handle_start_server(&self, input: &StartInput) -> LifecycleResult {
        GuestModule::handle_start_server(self, input)
    }

    fn handle_tick_client(&self, input: &TickInput) -> LifecycleResult {
        GuestModule::handle_tick_client(self, input)
    }

    fn handle_tick_server(&self, input: &TickInput) -> LifecycleResult {
        GuestModule::handle_tick_server(self, input)
    }

    fn handle_client_messages(&self, input: ClientMessageInput) -> ClientMessageResult {
        GuestModule::handle_client_messages(self, input)
    }

    fn handle_action(&self, input: ActionInput<'_>) -> ActionResult {
        GuestModule::handle_action(self, input)
    }

    fn handle_server_messages(&self, input: ServerMessageInput) -> ServerMessageResult {
        GuestModule::handle_server_messages(self, input)
    }

    fn handle_worldgen(&self, input: WorldGenCallInput) -> WorldGenCallResult {
        GuestModule::handle_worldgen(self, input)
    }

    fn handle_character_controller_init(
        &self,
        input: CharacterControllerInitInput,
    ) -> CharacterControllerInitResult {
        GuestModule::handle_character_controller_init(self, input)
    }

    fn handle_character_controller_step(
        &self,
        input: CharacterControllerStepInput,
    ) -> CharacterControllerStepResult {
        GuestModule::handle_character_controller_step(self, input)
    }

    fn handle_client_control_provider(
        &self,
        input: ClientControlSampleInput,
    ) -> ClientControlSampleResult {
        GuestModule::handle_client_control_provider(self, input)
    }
}

struct ActiveGuestSession<S> {
    info: RuntimeSessionInfo,
    state: S,
}

#[derive(Default)]
pub struct StatefulGuestSessionStore<S> {
    current: Option<ActiveGuestSession<S>>,
}

impl<S> StatefulGuestSessionStore<S> {
    #[must_use]
    pub const fn new() -> Self {
        Self { current: None }
    }

    fn ensure_current(
        &mut self,
        input: &StartInput,
        factory: StatefulSessionFactory<S>,
    ) -> &mut ActiveGuestSession<S> {
        let replace = self
            .current
            .as_ref()
            .is_none_or(|current| current.info != input.session);
        if replace {
            self.current = Some(ActiveGuestSession {
                info: input.session,
                state: factory(StartContext { input }),
            });
        }
        self.current
            .as_mut()
            .expect("stateful guest session must exist after ensure_current")
    }

    fn current_mut(&mut self, callback: &'static str) -> &mut ActiveGuestSession<S> {
        self.current.as_mut().unwrap_or_else(|| {
            panic!(
                "freven_guest_sdk stateful callback '{callback}' ran before start_client/start_server created a runtime session"
            )
        })
    }
}

pub struct StatefulGuestModule<S: 'static> {
    module: GuestModule,
    session_factory: StatefulSessionFactory<S>,
    session_store: &'static LocalKey<RefCell<StatefulGuestSessionStore<S>>>,
    on_start_client: Option<StatefulStartHandler<S>>,
    on_start_server: Option<StatefulStartHandler<S>>,
    on_tick_client: Option<StatefulTickHandler<S>>,
    on_tick_server: Option<StatefulTickHandler<S>>,
    on_client_messages: Option<StatefulClientMessageHandler<S>>,
    on_server_messages: Option<StatefulServerMessageHandler<S>>,
    actions: Vec<StatefulGuestAction<S>>,
}

impl<S: 'static> StatefulGuestModule<S> {
    #[must_use]
    pub fn new(
        guest_id: &'static str,
        session_factory: StatefulSessionFactory<S>,
        session_store: &'static LocalKey<RefCell<StatefulGuestSessionStore<S>>>,
    ) -> Self {
        Self {
            module: GuestModule::new(guest_id),
            session_factory,
            session_store,
            on_start_client: None,
            on_start_server: None,
            on_tick_client: None,
            on_tick_server: None,
            on_client_messages: None,
            on_server_messages: None,
            actions: Vec::new(),
        }
    }

    #[must_use]
    pub fn register_block(mut self, key: &'static str, def: BlockDescriptor) -> Self {
        self.module = self.module.register_block(key, def);
        self
    }

    #[must_use]
    pub fn register_component(mut self, key: &'static str, codec: ComponentCodec) -> Self {
        self.module = self.module.register_component(key, codec);
        self
    }

    #[must_use]
    pub fn register_message(mut self, key: &'static str, codec: MessageCodec) -> Self {
        self.module = self.module.register_message(key, codec);
        self
    }

    #[must_use]
    pub fn register_worldgen(mut self, key: &'static str) -> Self {
        self.module = self.module.register_worldgen(key);
        self
    }

    #[must_use]
    pub fn register_worldgen_handler(
        mut self,
        key: &'static str,
        handler: WorldGenHandler,
    ) -> Self {
        self.module = self.module.register_worldgen_handler(key, handler);
        self
    }

    #[must_use]
    pub fn register_character_controller(mut self, key: &'static str) -> Self {
        self.module = self.module.register_character_controller(key);
        self
    }

    #[must_use]
    pub fn register_character_controller_handler(
        mut self,
        key: &'static str,
        init: CharacterControllerInitHandler,
        step: CharacterControllerStepHandler,
    ) -> Self {
        self.module = self
            .module
            .register_character_controller_handler(key, init, step);
        self
    }

    #[must_use]
    pub fn register_client_control_provider(mut self, key: &'static str) -> Self {
        self.module = self.module.register_client_control_provider(key);
        self
    }

    #[must_use]
    pub fn register_client_control_provider_handler(
        mut self,
        key: &'static str,
        handler: ClientControlProviderHandler,
    ) -> Self {
        self.module = self
            .module
            .register_client_control_provider_handler(key, handler);
        self
    }

    #[must_use]
    pub fn register_channel(mut self, key: &'static str, config: ChannelConfig) -> Self {
        self.module = self.module.register_channel(key, config);
        self
    }

    #[must_use]
    pub fn declare_capability(mut self, key: &'static str) -> Self {
        self.module = self.module.declare_capability(key);
        self
    }

    #[must_use]
    pub fn on_start_client(mut self, handler: StatefulStartHandler<S>) -> Self {
        self.on_start_client = Some(handler);
        self
    }

    #[must_use]
    pub fn on_start_server(mut self, handler: StatefulStartHandler<S>) -> Self {
        self.on_start_server = Some(handler);
        self
    }

    #[must_use]
    pub fn on_tick_client(mut self, handler: StatefulTickHandler<S>) -> Self {
        self.on_tick_client = Some(handler);
        self
    }

    #[must_use]
    pub fn on_tick_server(mut self, handler: StatefulTickHandler<S>) -> Self {
        self.on_tick_server = Some(handler);
        self
    }

    #[must_use]
    pub fn on_client_messages(mut self, handler: StatefulClientMessageHandler<S>) -> Self {
        self.on_client_messages = Some(handler);
        self
    }

    #[must_use]
    pub fn on_server_messages(mut self, handler: StatefulServerMessageHandler<S>) -> Self {
        self.on_server_messages = Some(handler);
        self
    }

    #[must_use]
    pub fn action(
        mut self,
        key: &'static str,
        binding_id: u32,
        handler: StatefulActionHandler<S>,
    ) -> Self {
        assert_unique_key("action", key, self.actions.iter().map(|entry| entry.key));
        assert!(
            self.actions
                .iter()
                .all(|action| action.binding_id != binding_id),
            "freven_guest_sdk binding id {binding_id} was registered more than once"
        );
        self.actions.push(StatefulGuestAction {
            key,
            binding_id,
            handler,
        });
        self
    }

    fn lifecycle_hooks(&self) -> LifecycleHooks {
        LifecycleHooks {
            start_client: self.on_start_client.is_some(),
            start_server: self.on_start_server.is_some(),
            tick_client: self.on_tick_client.is_some(),
            tick_server: self.on_tick_server.is_some(),
        }
    }

    fn callbacks(&self) -> GuestCallbacks {
        GuestCallbacks {
            lifecycle: self.lifecycle_hooks(),
            action: !self.actions.is_empty(),
            messages: MessageHooks {
                client: self.on_client_messages.is_some(),
                server: self.on_server_messages.is_some(),
            },
            providers: ProviderHooks {
                world: WorldProviderHooks {
                    worldgen: !self.module.worldgen_handlers.is_empty(),
                },
                avatar: AvatarProviderHooks {
                    character_controller: !self.module.character_controller_handlers.is_empty(),
                    client_control_provider: !self
                        .module
                        .client_control_provider_handlers
                        .is_empty(),
                },
            },
        }
    }
}

impl<S: 'static> ExportedGuestModule for StatefulGuestModule<S> {
    fn description(&self) -> GuestDescription {
        GuestDescription {
            guest_id: self.module.guest_id.to_string(),
            registration: GuestRegistration {
                blocks: self.module.blocks.clone(),
                components: self.module.components.clone(),
                messages: self.module.messages.clone(),
                world: WorldGuestRegistration {
                    worldgen: self.module.worldgen.clone(),
                },
                avatar: AvatarGuestRegistration {
                    character_controllers: self.module.character_controllers.clone(),
                    client_control_providers: self.module.client_control_providers.clone(),
                },
                channels: self.module.channels.clone(),
                actions: self
                    .actions
                    .iter()
                    .map(|action| ActionDeclaration {
                        key: action.key.to_string(),
                        binding_id: action.binding_id,
                    })
                    .collect(),
                capabilities: self.module.capabilities.clone(),
            },
            callbacks: self.callbacks(),
        }
    }

    fn handle_start_client(&self, input: &StartInput) -> LifecycleResult {
        let Some(handler) = self.on_start_client else {
            return LifecycleResponse::default().finish();
        };
        self.session_store.with(|store| {
            let mut store = store.borrow_mut();
            let session = store.ensure_current(input, self.session_factory);
            handler(&mut session.state, StartContext { input })
        })
    }

    fn handle_start_server(&self, input: &StartInput) -> LifecycleResult {
        let Some(handler) = self.on_start_server else {
            return LifecycleResponse::default().finish();
        };
        self.session_store.with(|store| {
            let mut store = store.borrow_mut();
            let session = store.ensure_current(input, self.session_factory);
            handler(&mut session.state, StartContext { input })
        })
    }

    fn handle_tick_client(&self, input: &TickInput) -> LifecycleResult {
        let Some(handler) = self.on_tick_client else {
            return LifecycleResponse::default().finish();
        };
        self.session_store.with(|store| {
            let mut store = store.borrow_mut();
            let session = store.current_mut("tick_client");
            assert_eq!(session.info.side, RuntimeSessionSide::Client);
            handler(&mut session.state, TickContext { input })
        })
    }

    fn handle_tick_server(&self, input: &TickInput) -> LifecycleResult {
        let Some(handler) = self.on_tick_server else {
            return LifecycleResponse::default().finish();
        };
        self.session_store.with(|store| {
            let mut store = store.borrow_mut();
            let session = store.current_mut("tick_server");
            assert_eq!(session.info.side, RuntimeSessionSide::Server);
            handler(&mut session.state, TickContext { input })
        })
    }

    fn handle_client_messages(&self, input: ClientMessageInput) -> ClientMessageResult {
        let Some(handler) = self.on_client_messages else {
            return ClientMessageResponse::default().finish();
        };
        self.session_store.with(|store| {
            let mut store = store.borrow_mut();
            let session = store.current_mut("client_messages");
            assert_eq!(session.info.side, RuntimeSessionSide::Client);
            handler(&mut session.state, ClientMessageContext { input: &input }).finish()
        })
    }

    fn handle_action(&self, input: ActionInput<'_>) -> ActionResult {
        let Some(action) = self
            .actions
            .iter()
            .find(|action| action.binding_id == input.binding_id)
        else {
            return ActionResponse::rejected().finish();
        };
        self.session_store.with(|store| {
            let mut store = store.borrow_mut();
            let session = store.current_mut("action");
            (action.handler)(&mut session.state, ActionContext { input })
        })
    }

    fn handle_server_messages(&self, input: ServerMessageInput) -> ServerMessageResult {
        let Some(handler) = self.on_server_messages else {
            return ServerMessageResponse::default().finish();
        };
        self.session_store.with(|store| {
            let mut store = store.borrow_mut();
            let session = store.current_mut("server_messages");
            assert_eq!(session.info.side, RuntimeSessionSide::Server);
            handler(&mut session.state, ServerMessageContext { input: &input }).finish()
        })
    }

    fn handle_worldgen(&self, input: WorldGenCallInput) -> WorldGenCallResult {
        self.module.handle_worldgen(input)
    }

    fn handle_character_controller_init(
        &self,
        input: CharacterControllerInitInput,
    ) -> CharacterControllerInitResult {
        self.module.handle_character_controller_init(input)
    }

    fn handle_character_controller_step(
        &self,
        input: CharacterControllerStepInput,
    ) -> CharacterControllerStepResult {
        self.module.handle_character_controller_step(input)
    }

    fn handle_client_control_provider(
        &self,
        input: ClientControlSampleInput,
    ) -> ClientControlSampleResult {
        self.module.handle_client_control_provider(input)
    }
}

fn assert_unique_key<'a>(kind: &str, key: &'static str, existing: impl Iterator<Item = &'a str>) {
    assert!(
        !key.trim().is_empty(),
        "freven_guest_sdk {kind} key must not be empty"
    );
    assert!(
        existing.into_iter().all(|entry| entry != key),
        "freven_guest_sdk {kind} key '{key}' was registered more than once"
    );
}

struct GuestAction {
    key: &'static str,
    binding_id: u32,
    handler: ActionHandler,
}

struct StatefulGuestAction<S> {
    key: &'static str,
    binding_id: u32,
    handler: StatefulActionHandler<S>,
}

struct GuestWorldGen {
    key: &'static str,
    handler: WorldGenHandler,
}

struct GuestCharacterController {
    key: &'static str,
    init: CharacterControllerInitHandler,
    step: CharacterControllerStepHandler,
}

struct GuestClientControlProvider {
    key: &'static str,
    handler: ClientControlProviderHandler,
}

pub struct ActionContext<'a> {
    input: ActionInput<'a>,
}

impl<'a> ActionContext<'a> {
    #[must_use]
    pub fn input(&self) -> &ActionInput<'a> {
        &self.input
    }

    #[must_use]
    pub fn binding_id(&self) -> u32 {
        self.input.binding_id
    }

    #[must_use]
    pub fn player_id(&self) -> u64 {
        self.input.player_id
    }

    #[must_use]
    pub fn level_id(&self) -> u32 {
        self.input.level_id
    }

    #[must_use]
    pub fn stream_epoch(&self) -> u32 {
        self.input.stream_epoch
    }

    #[must_use]
    pub fn action_seq(&self) -> u32 {
        self.input.action_seq
    }

    #[must_use]
    pub fn at_input_seq(&self) -> u32 {
        self.input.at_input_seq
    }

    #[must_use]
    pub fn payload(&self) -> &'a [u8] {
        self.input.payload
    }

    pub fn decode_payload<T>(&self) -> Result<T, postcard::Error>
    where
        T: DeserializeOwned,
    {
        postcard::from_bytes(self.input.payload)
    }

    #[must_use]
    pub fn services(&self) -> RuntimeServices {
        RuntimeServices
    }
}

pub struct WorldGenContext<'a> {
    input: &'a WorldGenCallInput,
}

impl<'a> WorldGenContext<'a> {
    #[must_use]
    pub fn input(&self) -> &'a WorldGenCallInput {
        self.input
    }

    #[must_use]
    pub fn key(&self) -> &'a str {
        &self.input.key
    }

    #[must_use]
    pub fn init(&self) -> &'a WorldGenInit {
        &self.input.init
    }

    #[must_use]
    pub fn request(&self) -> &'a WorldGenRequest {
        &self.input.request
    }
}

pub struct CharacterControllerInitContext<'a> {
    input: &'a CharacterControllerInitInput,
}

impl<'a> CharacterControllerInitContext<'a> {
    #[must_use]
    pub fn input(&self) -> &'a CharacterControllerInitInput {
        self.input
    }

    #[must_use]
    pub fn key(&self) -> &'a str {
        &self.input.key
    }
}

pub struct CharacterControllerStepContext<'a> {
    input: &'a CharacterControllerStepInput,
}

impl<'a> CharacterControllerStepContext<'a> {
    #[must_use]
    pub fn input(&self) -> &'a CharacterControllerStepInput {
        self.input
    }

    #[must_use]
    pub fn key(&self) -> &'a str {
        &self.input.key
    }

    #[must_use]
    pub fn state(&self) -> CharacterState {
        self.input.state
    }

    #[must_use]
    pub fn controller_input(&self) -> &'a CharacterControllerInput {
        &self.input.input
    }

    #[must_use]
    pub fn dt_millis(&self) -> u32 {
        self.input.dt_millis
    }

    #[must_use]
    pub fn services(&self) -> RuntimeServices {
        RuntimeServices
    }
}

pub struct ClientControlProviderContext<'a> {
    input: &'a ClientControlSampleInput,
}

impl<'a> ClientControlProviderContext<'a> {
    #[must_use]
    pub fn input(&self) -> &'a ClientControlSampleInput {
        self.input
    }

    #[must_use]
    pub fn key(&self) -> &'a str {
        &self.input.key
    }

    #[must_use]
    pub fn services(&self) -> RuntimeServices {
        RuntimeServices
    }
}

pub struct StartContext<'a> {
    input: &'a StartInput,
}

impl<'a> StartContext<'a> {
    #[must_use]
    pub fn input(&self) -> &'a StartInput {
        self.input
    }

    #[must_use]
    pub fn session(&self) -> RuntimeSessionInfo {
        self.input.session
    }

    #[must_use]
    pub fn experience_id(&self) -> &'a str {
        &self.input.experience_id
    }

    #[must_use]
    pub fn mod_id(&self) -> &'a str {
        &self.input.mod_id
    }

    #[must_use]
    pub fn services(&self) -> RuntimeServices {
        RuntimeServices
    }
}

pub struct TickContext<'a> {
    input: &'a TickInput,
}

impl<'a> TickContext<'a> {
    #[must_use]
    pub fn input(&self) -> &'a TickInput {
        self.input
    }

    #[must_use]
    pub fn tick(&self) -> u64 {
        self.input.tick
    }

    #[must_use]
    pub fn dt_millis(&self) -> u32 {
        self.input.dt_millis
    }

    #[must_use]
    pub fn services(&self) -> RuntimeServices {
        RuntimeServices
    }
}

pub trait StartInputExt {
    fn config_text(&self) -> &str;
    fn config_typed<T>(&self) -> Result<T, toml::de::Error>
    where
        T: DeserializeOwned;
}

impl StartInputExt for StartInput {
    fn config_text(&self) -> &str {
        &self.config.text
    }

    fn config_typed<T>(&self) -> Result<T, toml::de::Error>
    where
        T: DeserializeOwned,
    {
        match self.config.format {
            ModConfigFormat::Toml => toml::from_str(&self.config.text),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeQuerySupport<T> {
    Supported(T),
    Unsupported,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimeServices;

impl RuntimeServices {
    pub fn log(self, level: LogLevel, message: impl Into<String>) {
        let _ = runtime_service_call(WorldServiceRequest::Observability(
            RuntimeObservabilityRequest::Log(LogPayload {
                level,
                message: message.into(),
            }),
        ));
    }

    #[must_use]
    pub fn authoritative_block(
        self,
        pos: (i32, i32, i32),
    ) -> RuntimeQuerySupport<Option<BlockRuntimeId>> {
        match runtime_service_call(WorldServiceRequest::Block(BlockServiceRequest::Query(
            BlockQueryRequest::AuthoritativeBlock { pos },
        ))) {
            WorldServiceResponse::Block(BlockServiceResponse::Query(
                BlockQueryResponse::AuthoritativeBlock(value),
            )) => RuntimeQuerySupport::Supported(value),
            WorldServiceResponse::Unsupported => RuntimeQuerySupport::Unsupported,
            other => {
                debug_assert!(
                    false,
                    "unexpected response for AuthoritativeBlock query: {:?}",
                    other
                );
                RuntimeQuerySupport::Unsupported
            }
        }
    }

    #[must_use]
    pub fn block_id_by_key(self, key: &str) -> Option<BlockRuntimeId> {
        match runtime_service_call(WorldServiceRequest::Block(BlockServiceRequest::Query(
            BlockQueryRequest::BlockIdByKey {
                key: key.to_string(),
            },
        ))) {
            WorldServiceResponse::Block(BlockServiceResponse::Query(
                BlockQueryResponse::BlockIdByKey(value),
            )) => value,
            _ => None,
        }
    }

    #[must_use]
    pub fn player_position(self, player_id: u64) -> Option<[f32; 3]> {
        match runtime_service_call(WorldServiceRequest::Query(
            WorldQueryRequest::PlayerPosition { player_id },
        )) {
            WorldServiceResponse::Query(WorldQueryResponse::PlayerPosition(value)) => value,
            _ => None,
        }
    }

    #[must_use]
    pub fn player_display_name(self, player_id: u64) -> Option<String> {
        match runtime_service_call(WorldServiceRequest::Query(
            WorldQueryRequest::PlayerDisplayName { player_id },
        )) {
            WorldServiceResponse::Query(WorldQueryResponse::PlayerDisplayName(value)) => value,
            _ => None,
        }
    }

    #[must_use]
    pub fn client_visible_block(self, pos: (i32, i32, i32)) -> Option<BlockRuntimeId> {
        match runtime_service_call(WorldServiceRequest::Block(
            BlockServiceRequest::ClientQuery(BlockClientQueryRequest::ClientVisibleBlock { pos }),
        )) {
            WorldServiceResponse::Block(BlockServiceResponse::ClientQuery(
                BlockClientQueryResponse::ClientVisibleBlock(value),
            )) => value,
            _ => None,
        }
    }

    #[must_use]
    pub fn client_player_views(self) -> Vec<ClientPlayerView> {
        match runtime_service_call(WorldServiceRequest::ClientVisibility(
            ClientVisibilityRequest::ClientPlayerViews,
        )) {
            WorldServiceResponse::ClientVisibility(
                ClientVisibilityResponse::ClientPlayerViews(value),
            ) => value,
            _ => Vec::new(),
        }
    }

    #[must_use]
    pub fn client_world_to_screen(self, world_pos_m: (f32, f32, f32)) -> Option<(i32, i32)> {
        match runtime_service_call(WorldServiceRequest::ClientVisibility(
            ClientVisibilityRequest::ClientWorldToScreen { world_pos_m },
        )) {
            WorldServiceResponse::ClientVisibility(
                ClientVisibilityResponse::ClientWorldToScreen(value),
            ) => value,
            _ => None,
        }
    }

    #[must_use]
    pub fn client_active_level(self) -> Option<RuntimeLevelRef> {
        match runtime_service_call(WorldServiceRequest::ClientVisibility(
            ClientVisibilityRequest::ClientActiveLevel,
        )) {
            WorldServiceResponse::ClientVisibility(
                ClientVisibilityResponse::ClientActiveLevel(value),
            ) => value,
            _ => None,
        }
    }

    #[must_use]
    pub fn client_next_input_seq(self) -> Option<u32> {
        match runtime_service_call(WorldServiceRequest::ClientVisibility(
            ClientVisibilityRequest::ClientNextInputSeq,
        )) {
            WorldServiceResponse::ClientVisibility(
                ClientVisibilityResponse::ClientNextInputSeq(value),
            ) => value,
            _ => None,
        }
    }

    #[must_use]
    pub fn server_player_connected(self, player_id: u64) -> Option<bool> {
        match runtime_service_call(WorldServiceRequest::Session(
            WorldSessionRequest::ServerPlayerConnected { player_id },
        )) {
            WorldServiceResponse::Session(WorldSessionResponse::ServerPlayerConnected(value)) => {
                value
            }
            _ => None,
        }
    }

    #[must_use]
    pub fn bind_mouse_button(self, button: ClientMouseButton, owner: &str) -> bool {
        matches!(
            runtime_service_call(WorldServiceRequest::ClientControl(
                RuntimeClientControlRequest::BindMouseButton {
                    button,
                    owner: owner.to_string(),
                },
            )),
            WorldServiceResponse::ClientControlBool(true)
        )
    }

    #[must_use]
    pub fn bind_key(self, key: ClientKeyCode, owner: &str) -> bool {
        matches!(
            runtime_service_call(WorldServiceRequest::ClientControl(
                RuntimeClientControlRequest::BindKey {
                    key,
                    owner: owner.to_string(),
                },
            )),
            WorldServiceResponse::ClientControlBool(true)
        )
    }

    #[must_use]
    pub fn mouse_button_down(self, button: ClientMouseButton, owner: &str) -> bool {
        matches!(
            runtime_service_call(WorldServiceRequest::ClientControl(
                RuntimeClientControlRequest::MouseButtonDown {
                    button,
                    owner: owner.to_string(),
                },
            )),
            WorldServiceResponse::ClientControlBool(true)
        )
    }

    #[must_use]
    pub fn key_down(self, key: ClientKeyCode, owner: &str) -> bool {
        matches!(
            runtime_service_call(WorldServiceRequest::ClientControl(
                RuntimeClientControlRequest::KeyDown {
                    key,
                    owner: owner.to_string(),
                },
            )),
            WorldServiceResponse::ClientControlBool(true)
        )
    }

    #[must_use]
    pub fn mouse_delta(self) -> (i32, i32) {
        match runtime_service_call(WorldServiceRequest::ClientControl(
            RuntimeClientControlRequest::MouseDelta,
        )) {
            WorldServiceResponse::ClientControlMouseDelta(value) => value,
            _ => (0, 0),
        }
    }

    #[must_use]
    pub fn cursor_locked(self) -> bool {
        matches!(
            runtime_service_call(WorldServiceRequest::ClientControl(
                RuntimeClientControlRequest::CursorLocked,
            )),
            WorldServiceResponse::ClientControlBool(true)
        )
    }

    #[must_use]
    pub fn view_angles_deg_mdeg(self) -> (i32, i32) {
        match runtime_service_call(WorldServiceRequest::ClientControl(
            RuntimeClientControlRequest::ViewAnglesDegMdeg,
        )) {
            WorldServiceResponse::ClientControlViewAnglesDegMdeg(value) => value,
            _ => (0, 0),
        }
    }

    #[must_use]
    pub fn is_solid_world_collision(self, wx: i32, wy: i32, wz: i32) -> bool {
        matches!(
            runtime_service_call(WorldServiceRequest::CharacterPhysics(
                RuntimeCharacterPhysicsRequest::IsSolidWorldCollision { wx, wy, wz },
            )),
            WorldServiceResponse::CharacterPhysicsIsSolidWorldCollision(true)
        )
    }

    #[must_use]
    pub fn sweep_aabb(self, half_extents: [f32; 3], from: [f32; 3], to: [f32; 3]) -> SweepHit {
        match runtime_service_call(WorldServiceRequest::CharacterPhysics(
            RuntimeCharacterPhysicsRequest::SweepAabb {
                half_extents,
                from,
                to,
            },
        )) {
            WorldServiceResponse::CharacterPhysicsSweepAabb(value) => value,
            _ => SweepHit {
                hit: false,
                toi: 1.0,
                normal: [0.0, 0.0, 0.0],
            },
        }
    }

    #[must_use]
    pub fn move_aabb_terrain(
        self,
        half_extents: [f32; 3],
        pos: [f32; 3],
        motion: [f32; 3],
        cfg: KinematicMoveConfig,
    ) -> KinematicMoveResult {
        match runtime_service_call(WorldServiceRequest::CharacterPhysics(
            RuntimeCharacterPhysicsRequest::MoveAabbTerrain {
                half_extents,
                pos,
                motion,
                cfg,
            },
        )) {
            WorldServiceResponse::CharacterPhysicsMoveAabbTerrain(value) => value,
            _ => KinematicMoveResult {
                pos,
                applied_motion: [0.0, 0.0, 0.0],
                hit_x: false,
                hit_y: false,
                hit_z: false,
                hit_ground: false,
                started_overlapping: false,
                collision_incomplete: true,
            },
        }
    }
}

/// Emit a log message through the canonical guest observability service.
///
/// Prefer [`log_debug!`], [`log_info!`], [`log_warn!`], [`log_error!`] over
/// calling this directly.
#[doc(hidden)]
pub fn __guest_emit_log(level: LogLevel, args: ::core::fmt::Arguments<'_>) {
    RuntimeServices.log(level, ::alloc::format!("{args}"));
}

/// Log a debug message from a guest mod.
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::__guest_emit_log($crate::__private::LogLevel::Debug, ::core::format_args!($($arg)*));
    };
}

/// Log an info message from a guest mod.
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::__guest_emit_log($crate::__private::LogLevel::Info, ::core::format_args!($($arg)*));
    };
}

/// Log a warning from a guest mod.
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::__guest_emit_log($crate::__private::LogLevel::Warn, ::core::format_args!($($arg)*));
    };
}

/// Log an error from a guest mod.
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::__guest_emit_log($crate::__private::LogLevel::Error, ::core::format_args!($($arg)*));
    };
}

pub struct ActionResponse;

pub struct AppliedActionResponse {
    output: RuntimeOutput,
}

pub struct RejectedActionResponse {
    output: RuntimeOutput,
}

impl ActionResponse {
    #[must_use]
    pub fn applied() -> AppliedActionResponse {
        AppliedActionResponse {
            output: RuntimeOutput::default(),
        }
    }

    #[must_use]
    pub fn rejected() -> RejectedActionResponse {
        RejectedActionResponse {
            output: RuntimeOutput::default(),
        }
    }
}

impl AppliedActionResponse {
    #[must_use]
    pub fn push_block_mutation(mut self, command: BlockMutation) -> Self {
        self.output.blocks.mutations.push(command);
        self
    }

    #[must_use]
    pub fn set_block(self, pos: (i32, i32, i32), block_id: BlockRuntimeId) -> Self {
        self.push_block_mutation(BlockMutation::SetBlock {
            pos,
            block_id,
            expected_old: None,
        })
    }

    #[must_use]
    pub fn set_block_if(
        self,
        pos: (i32, i32, i32),
        expected_old: BlockRuntimeId,
        block_id: BlockRuntimeId,
    ) -> Self {
        self.push_block_mutation(BlockMutation::SetBlock {
            pos,
            block_id,
            expected_old: Some(expected_old),
        })
    }

    #[must_use]
    pub fn send_client(mut self, message: ClientOutboundMessage) -> Self {
        self.output.messages.client.push(message);
        self
    }

    #[must_use]
    pub fn send_server(mut self, message: ServerOutboundMessage) -> Self {
        self.output.messages.server.push(message);
        self
    }

    #[must_use]
    pub fn finish(self) -> ActionResult {
        ActionResult {
            outcome: ActionOutcome::Applied,
            output: self.output,
        }
    }
}

impl RejectedActionResponse {
    #[must_use]
    pub fn send_client(mut self, message: ClientOutboundMessage) -> Self {
        self.output.messages.client.push(message);
        self
    }

    #[must_use]
    pub fn send_server(mut self, message: ServerOutboundMessage) -> Self {
        self.output.messages.server.push(message);
        self
    }

    #[must_use]
    pub fn finish(self) -> ActionResult {
        ActionResult {
            outcome: ActionOutcome::Rejected,
            output: self.output,
        }
    }
}

pub struct ClientMessageContext<'a> {
    input: &'a ClientMessageInput,
}

impl<'a> ClientMessageContext<'a> {
    #[must_use]
    pub fn tick(&self) -> u64 {
        self.input.tick
    }

    #[must_use]
    pub fn dt_millis(&self) -> u32 {
        self.input.dt_millis
    }

    #[must_use]
    pub fn messages(&self) -> &'a [ClientInboundMessage] {
        &self.input.messages
    }

    #[must_use]
    pub fn services(&self) -> RuntimeServices {
        RuntimeServices
    }
}

#[derive(Default)]
pub struct ClientMessageResponse {
    output: RuntimeOutput,
}

impl ClientMessageResponse {
    #[must_use]
    pub fn send(mut self, message: ClientOutboundMessage) -> Self {
        self.output.messages.client.push(message);
        self
    }

    #[must_use]
    pub fn send_to(mut self, message: ServerOutboundMessage) -> Self {
        self.output.messages.server.push(message);
        self
    }

    #[must_use]
    pub fn set_block(mut self, pos: (i32, i32, i32), block_id: BlockRuntimeId) -> Self {
        self.output.blocks.mutations.push(BlockMutation::SetBlock {
            pos,
            block_id,
            expected_old: None,
        });
        self
    }

    #[must_use]
    pub fn finish(self) -> ClientMessageResult {
        ClientMessageResult {
            output: self.output,
        }
    }
}

pub struct ServerMessageContext<'a> {
    input: &'a ServerMessageInput,
}

impl<'a> ServerMessageContext<'a> {
    #[must_use]
    pub fn tick(&self) -> u64 {
        self.input.tick
    }

    #[must_use]
    pub fn dt_millis(&self) -> u32 {
        self.input.dt_millis
    }

    #[must_use]
    pub fn messages(&self) -> &'a [ServerInboundMessage] {
        &self.input.messages
    }

    #[must_use]
    pub fn services(&self) -> RuntimeServices {
        RuntimeServices
    }
}

#[derive(Default)]
pub struct ServerMessageResponse {
    output: RuntimeOutput,
}

impl ServerMessageResponse {
    #[must_use]
    pub fn send_to(mut self, message: ServerOutboundMessage) -> Self {
        self.output.messages.server.push(message);
        self
    }

    #[must_use]
    pub fn send(mut self, message: ClientOutboundMessage) -> Self {
        self.output.messages.client.push(message);
        self
    }

    #[must_use]
    pub fn set_block(mut self, pos: (i32, i32, i32), block_id: BlockRuntimeId) -> Self {
        self.output.blocks.mutations.push(BlockMutation::SetBlock {
            pos,
            block_id,
            expected_old: None,
        });
        self
    }

    #[must_use]
    pub fn finish(self) -> ServerMessageResult {
        ServerMessageResult {
            output: self.output,
        }
    }
}

#[derive(Default)]
pub struct LifecycleResponse {
    output: RuntimeOutput,
}

impl LifecycleResponse {
    #[must_use]
    pub fn send(mut self, message: ClientOutboundMessage) -> Self {
        self.output.messages.client.push(message);
        self
    }

    #[must_use]
    pub fn send_to(mut self, message: ServerOutboundMessage) -> Self {
        self.output.messages.server.push(message);
        self
    }

    #[must_use]
    pub fn set_block(mut self, pos: (i32, i32, i32), block_id: BlockRuntimeId) -> Self {
        self.output.blocks.mutations.push(BlockMutation::SetBlock {
            pos,
            block_id,
            expected_old: None,
        });
        self
    }

    #[must_use]
    pub fn finish(self) -> LifecycleResult {
        LifecycleResult {
            output: self.output,
        }
    }
}

#[cfg(test)]
type TestRuntimeServiceHook = dyn FnMut(WorldServiceRequest) -> WorldServiceResponse;

#[cfg(test)]
type TestRuntimeServiceHookSlot = RefCell<Option<Box<TestRuntimeServiceHook>>>;

#[cfg(test)]
thread_local! {
    static TEST_RUNTIME_SERVICE_HOOK: TestRuntimeServiceHookSlot =
        RefCell::new(None);
}

#[cfg(test)]
struct TestRuntimeServiceHookGuard;

#[cfg(test)]
impl Drop for TestRuntimeServiceHookGuard {
    fn drop(&mut self) {
        TEST_RUNTIME_SERVICE_HOOK.with(|slot| {
            let _ = slot.replace(None);
        });
    }
}

#[cfg(test)]
fn install_test_runtime_service_hook(
    hook: impl FnMut(WorldServiceRequest) -> WorldServiceResponse + 'static,
) -> TestRuntimeServiceHookGuard {
    TEST_RUNTIME_SERVICE_HOOK.with(|slot| {
        let previous = slot.replace(Some(Box::new(hook)));
        assert!(
            previous.is_none(),
            "nested test runtime service hook is not supported"
        );
    });
    TestRuntimeServiceHookGuard
}

fn runtime_service_call(request: WorldServiceRequest) -> WorldServiceResponse {
    #[cfg(test)]
    if let Some(response) = TEST_RUNTIME_SERVICE_HOOK.with(|slot| {
        let mut slot = slot.borrow_mut();
        slot.as_mut().map(|hook| hook(request.clone()))
    }) {
        return response;
    }

    let request_bytes =
        postcard::to_allocvec(&request).expect("runtime service request encoding must succeed");
    let mut response = vec![0u8; 64 * 1024];

    let len = if cfg!(target_arch = "wasm32") {
        wasm_runtime_service_call(&request_bytes, &mut response)
    } else {
        native_runtime_service_call(&request_bytes, &mut response)
    };

    let Some(len) = len else {
        return WorldServiceResponse::Unsupported;
    };

    postcard::from_bytes(&response[..len]).expect("runtime service response decoding must succeed")
}

#[cfg(target_arch = "wasm32")]
fn wasm_runtime_service_call(request: &[u8], response: &mut [u8]) -> Option<usize> {
    unsafe extern "C" {
        fn freven_guest_host_service_call(
            req_ptr: u32,
            req_len: u32,
            resp_ptr: u32,
            resp_cap: u32,
        ) -> u32;
    }

    let req_ptr = request.as_ptr() as usize as u32;
    let req_len = u32::try_from(request.len()).ok()?;
    let resp_ptr = response.as_mut_ptr() as usize as u32;
    let resp_cap = u32::try_from(response.len()).ok()?;
    let len = unsafe { freven_guest_host_service_call(req_ptr, req_len, resp_ptr, resp_cap) };
    if len == u32::MAX {
        return None;
    }
    Some(len as usize)
}

#[cfg(not(target_arch = "wasm32"))]
fn wasm_runtime_service_call(_request: &[u8], _response: &mut [u8]) -> Option<usize> {
    None
}

fn native_runtime_service_call(request: &[u8], response: &mut [u8]) -> Option<usize> {
    NATIVE_RUNTIME_BRIDGE.with(|bridge| {
        let bridge = *bridge.borrow();
        let call = bridge.call?;
        let len = unsafe {
            call(
                bridge.ctx,
                request.as_ptr(),
                request.len(),
                response.as_mut_ptr(),
                response.len(),
            )
        };
        if len == usize::MAX || len > response.len() {
            None
        } else {
            Some(len)
        }
    })
}

#[doc(hidden)]
pub mod __private {
    use super::*;

    pub type ChannelConfig = freven_guest::ChannelConfig;
    pub type ChannelDirection = freven_guest::ChannelDirection;
    pub type ChannelOrdering = freven_guest::ChannelOrdering;
    pub type ChannelReliability = freven_guest::ChannelReliability;
    pub type LifecycleHooks = freven_guest::LifecycleHooks;
    pub type LogLevel = freven_guest::LogLevel;
    pub type MessageHooks = freven_guest::MessageHooks;
    pub const GUEST_CONTRACT_VERSION_1: u32 = freven_guest::GUEST_CONTRACT_VERSION_1;

    fn module_negotiate_bytes(module: &impl ExportedGuestModule, input: &[u8]) -> Vec<u8> {
        if !input.is_empty() {
            let request: NegotiationRequest =
                postcard::from_bytes(input).expect("valid negotiation request");
            assert!(
                request
                    .supported_contract_versions
                    .contains(&GUEST_CONTRACT_VERSION_1)
            );
        }

        let response = NegotiationResponse {
            selected_contract_version: GUEST_CONTRACT_VERSION_1,
            description: module.description(),
        };
        postcard::to_allocvec(&response).expect("guest encoding must succeed")
    }

    fn module_start_client_bytes(module: &impl ExportedGuestModule, input: &[u8]) -> Vec<u8> {
        let input = decode_default_input::<StartInput>(input);
        postcard::to_allocvec(&module.handle_start_client(&input))
            .expect("guest encoding must succeed")
    }

    fn module_start_server_bytes(module: &impl ExportedGuestModule, input: &[u8]) -> Vec<u8> {
        let input = decode_default_input::<StartInput>(input);
        postcard::to_allocvec(&module.handle_start_server(&input))
            .expect("guest encoding must succeed")
    }

    fn module_tick_client_bytes(module: &impl ExportedGuestModule, input: &[u8]) -> Vec<u8> {
        let input = decode_required_input::<TickInput>(input);
        postcard::to_allocvec(&module.handle_tick_client(&input))
            .expect("guest encoding must succeed")
    }

    fn module_tick_server_bytes(module: &impl ExportedGuestModule, input: &[u8]) -> Vec<u8> {
        let input = decode_required_input::<TickInput>(input);
        postcard::to_allocvec(&module.handle_tick_server(&input))
            .expect("guest encoding must succeed")
    }

    fn module_handle_action_bytes(module: &impl ExportedGuestModule, input: &[u8]) -> Vec<u8> {
        assert!(!input.is_empty(), "guest input must not be empty");
        let input: ActionInput<'_> = postcard::from_bytes(input).expect("valid action input");

        let result = module.handle_action(input);
        postcard::to_allocvec(&result).expect("guest encoding must succeed")
    }

    fn module_client_messages_bytes(module: &impl ExportedGuestModule, input: &[u8]) -> Vec<u8> {
        let input = decode_required_input::<ClientMessageInput>(input);
        postcard::to_allocvec(&module.handle_client_messages(input))
            .expect("guest encoding must succeed")
    }

    fn module_server_messages_bytes(module: &impl ExportedGuestModule, input: &[u8]) -> Vec<u8> {
        let input = decode_required_input::<ServerMessageInput>(input);
        postcard::to_allocvec(&module.handle_server_messages(input))
            .expect("guest encoding must succeed")
    }

    fn module_worldgen_bytes(module: &impl ExportedGuestModule, input: &[u8]) -> Vec<u8> {
        let input = decode_required_input::<WorldGenCallInput>(input);
        postcard::to_allocvec(&module.handle_worldgen(input)).expect("guest encoding must succeed")
    }

    fn module_character_controller_init_bytes(
        module: &impl ExportedGuestModule,
        input: &[u8],
    ) -> Vec<u8> {
        let input = decode_required_input::<CharacterControllerInitInput>(input);
        postcard::to_allocvec(&module.handle_character_controller_init(input))
            .expect("guest encoding must succeed")
    }

    fn module_character_controller_step_bytes(
        module: &impl ExportedGuestModule,
        input: &[u8],
    ) -> Vec<u8> {
        let input = decode_required_input::<CharacterControllerStepInput>(input);
        postcard::to_allocvec(&module.handle_character_controller_step(input))
            .expect("guest encoding must succeed")
    }

    fn module_client_control_provider_bytes(
        module: &impl ExportedGuestModule,
        input: &[u8],
    ) -> Vec<u8> {
        let input = decode_required_input::<ClientControlSampleInput>(input);
        postcard::to_allocvec(&module.handle_client_control_provider(input))
            .expect("guest encoding must succeed")
    }

    pub fn wasm_guest_alloc(size: u32) -> u32 {
        let mut buf = Vec::<u8>::with_capacity(size as usize);
        let ptr = buf.as_mut_ptr();
        core::mem::forget(buf);
        ptr as usize as u32
    }

    pub fn wasm_guest_dealloc(ptr: u32, size: u32) {
        if ptr == 0 {
            return;
        }
        unsafe {
            let _ = Vec::from_raw_parts(ptr as usize as *mut u8, size as usize, size as usize);
        }
    }

    pub fn wasm_guest_negotiate(module: &impl ExportedGuestModule, ptr: u32, len: u32) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_negotiate_bytes(module, input))
        })
    }

    pub fn wasm_guest_start_client(module: &impl ExportedGuestModule, ptr: u32, len: u32) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_start_client_bytes(module, input))
        })
    }

    pub fn wasm_guest_start_server(module: &impl ExportedGuestModule, ptr: u32, len: u32) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_start_server_bytes(module, input))
        })
    }

    pub fn wasm_guest_tick_client(module: &impl ExportedGuestModule, ptr: u32, len: u32) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_tick_client_bytes(module, input))
        })
    }

    pub fn wasm_guest_tick_server(module: &impl ExportedGuestModule, ptr: u32, len: u32) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_tick_server_bytes(module, input))
        })
    }

    pub fn wasm_guest_handle_action(module: &impl ExportedGuestModule, ptr: u32, len: u32) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_handle_action_bytes(module, input))
        })
    }

    pub fn wasm_guest_client_messages(
        module: &impl ExportedGuestModule,
        ptr: u32,
        len: u32,
    ) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_client_messages_bytes(module, input))
        })
    }

    pub fn wasm_guest_server_messages(
        module: &impl ExportedGuestModule,
        ptr: u32,
        len: u32,
    ) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_server_messages_bytes(module, input))
        })
    }

    pub fn wasm_guest_worldgen(module: &impl ExportedGuestModule, ptr: u32, len: u32) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_worldgen_bytes(module, input))
        })
    }

    pub fn wasm_guest_character_controller_init(
        module: &impl ExportedGuestModule,
        ptr: u32,
        len: u32,
    ) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_character_controller_init_bytes(module, input))
        })
    }

    pub fn wasm_guest_character_controller_step(
        module: &impl ExportedGuestModule,
        ptr: u32,
        len: u32,
    ) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_character_controller_step_bytes(module, input))
        })
    }

    pub fn wasm_guest_client_control_provider(
        module: &impl ExportedGuestModule,
        ptr: u32,
        len: u32,
    ) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_client_control_provider_bytes(module, input))
        })
    }

    pub fn native_guest_alloc(size: usize) -> *mut u8 {
        if size == 0 {
            return core::ptr::null_mut();
        }

        let mut boxed = alloc::vec![0u8; size].into_boxed_slice();
        let ptr = boxed.as_mut_ptr();
        let _raw = alloc::boxed::Box::into_raw(boxed);
        ptr
    }

    pub fn native_guest_dealloc(buffer: NativeGuestBuffer) {
        if buffer.ptr.is_null() || buffer.len == 0 {
            return;
        }

        unsafe {
            let slice_ptr = core::ptr::slice_from_raw_parts_mut(buffer.ptr, buffer.len);
            drop(alloc::boxed::Box::from_raw(slice_ptr));
        }
    }

    pub fn native_guest_set_runtime_bridge(bridge: NativeRuntimeBridge) {
        NATIVE_RUNTIME_BRIDGE.with(|slot| {
            *slot.borrow_mut() = bridge;
        });
    }

    pub fn native_guest_negotiate(
        module: &impl ExportedGuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_negotiate_bytes(module, input))
        })
    }

    pub fn native_guest_start_client(
        module: &impl ExportedGuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_start_client_bytes(module, input))
        })
    }

    pub fn native_guest_start_server(
        module: &impl ExportedGuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_start_server_bytes(module, input))
        })
    }

    pub fn native_guest_tick_client(
        module: &impl ExportedGuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_tick_client_bytes(module, input))
        })
    }

    pub fn native_guest_tick_server(
        module: &impl ExportedGuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_tick_server_bytes(module, input))
        })
    }

    pub fn native_guest_handle_action(
        module: &impl ExportedGuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_handle_action_bytes(module, input))
        })
    }

    pub fn native_guest_client_messages(
        module: &impl ExportedGuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_client_messages_bytes(module, input))
        })
    }

    pub fn native_guest_server_messages(
        module: &impl ExportedGuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_server_messages_bytes(module, input))
        })
    }

    pub fn native_guest_worldgen(
        module: &impl ExportedGuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_worldgen_bytes(module, input))
        })
    }

    pub fn native_guest_character_controller_init(
        module: &impl ExportedGuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_character_controller_init_bytes(module, input))
        })
    }

    pub fn native_guest_character_controller_step(
        module: &impl ExportedGuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_character_controller_step_bytes(module, input))
        })
    }

    pub fn native_guest_client_control_provider(
        module: &impl ExportedGuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_client_control_provider_bytes(module, input))
        })
    }

    fn decode_default_input<T>(bytes: &[u8]) -> T
    where
        T: Default + serde::de::DeserializeOwned,
    {
        if bytes.is_empty() {
            return T::default();
        }
        postcard::from_bytes(bytes).expect("valid guest input")
    }

    fn decode_required_input<T>(bytes: &[u8]) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        assert!(!bytes.is_empty(), "guest input must not be empty");
        postcard::from_bytes(bytes).expect("valid guest input")
    }

    fn with_wasm_input_bytes<R>(ptr: u32, len: u32, f: impl FnOnce(&[u8]) -> R) -> R {
        if len == 0 {
            return f(&[]);
        }

        let bytes = unsafe { core::slice::from_raw_parts(ptr as usize as *const u8, len as usize) };
        f(bytes)
    }

    fn with_native_input_bytes<R>(input: NativeGuestInput, f: impl FnOnce(&[u8]) -> R) -> R {
        if input.len == 0 {
            return f(&[]);
        }

        let bytes = unsafe { core::slice::from_raw_parts(input.ptr, input.len) };
        f(bytes)
    }

    fn encode_to_wasm_guest(bytes: &[u8]) -> u64 {
        let len = u32::try_from(bytes.len()).expect("guest buffer length must fit u32");
        let ptr = wasm_guest_alloc(len);
        unsafe {
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as usize as *mut u8, bytes.len());
        }
        (u64::from(ptr) << 32) | u64::from(len)
    }

    fn encode_to_native_guest(bytes: Vec<u8>) -> NativeGuestBuffer {
        if bytes.is_empty() {
            return NativeGuestBuffer::empty();
        }

        let mut boxed = bytes.into_boxed_slice();
        let len = boxed.len();
        let ptr = boxed.as_mut_ptr();
        let _raw = alloc::boxed::Box::into_raw(boxed);

        NativeGuestBuffer { ptr, len }
    }

    pub fn assert_export_surface(
        module: &impl ExportedGuestModule,
        lifecycle: LifecycleHooks,
        action: bool,
        messages: MessageHooks,
        providers: ProviderHooks,
    ) {
        let callbacks = module.description().callbacks;
        assert_eq!(
            callbacks.lifecycle, lifecycle,
            "freven_guest_sdk export lifecycle does not match GuestModule::description()",
        );
        assert_eq!(
            callbacks.action, action,
            "freven_guest_sdk action export does not match GuestModule::description()",
        );
        assert_eq!(
            callbacks.messages, messages,
            "freven_guest_sdk message export surface does not match GuestModule::description()",
        );
        assert_eq!(
            callbacks.providers, providers,
            "freven_guest_sdk provider export surface does not match GuestModule::description()",
        );
    }
}

#[macro_export]
macro_rules! export_wasm_guest {
    (
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),* $(,)?])?
        $(, actions: $actions:tt)?
        $(, client_messages: $client_messages:tt)?
        $(, server_messages: $server_messages:tt)?
        $(, worldgen: $worldgen:tt)?
        $(, character_controller: $character_controller:tt)?
        $(, client_control_provider: $client_control_provider:tt)?
        $(,)?
    ) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_alloc(size: u32) -> u32 {
            $crate::__private::wasm_guest_alloc(size)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_dealloc(ptr: u32, size: u32) {
            $crate::__private::wasm_guest_dealloc(ptr, size)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_negotiate(ptr: u32, len: u32) -> u64 {
            let module = $factory();
            $crate::__private::assert_export_surface(
                &module,
                $crate::export_wasm_guest!(@lifecycle_struct $($($lifecycle),*)?),
                $crate::export_wasm_guest!(@bool $($actions)?),
                $crate::__private::MessageHooks {
                    client: $crate::export_wasm_guest!(@bool $($client_messages)?),
                    server: $crate::export_wasm_guest!(@bool $($server_messages)?),
                },
                $crate::ProviderHooks {
                    world: $crate::WorldProviderHooks {
                        worldgen: $crate::export_wasm_guest!(@bool $($worldgen)?),
                    },
                    avatar: $crate::AvatarProviderHooks {
                        character_controller: $crate::export_wasm_guest!(@bool $($character_controller)?),
                        client_control_provider: $crate::export_wasm_guest!(@bool $($client_control_provider)?),
                    },
                },
            );
            $crate::__private::wasm_guest_negotiate(&module, ptr, len)
        }

        $crate::export_wasm_guest!(@maybe_export $factory, start_client, $($($lifecycle),*)?);
        $crate::export_wasm_guest!(@maybe_export $factory, start_server, $($($lifecycle),*)?);
        $crate::export_wasm_guest!(@maybe_export $factory, tick_client, $($($lifecycle),*)?);
        $crate::export_wasm_guest!(@maybe_export $factory, tick_server, $($($lifecycle),*)?);
        $crate::export_wasm_guest!(@maybe_export_action $factory, $($actions)?);
        $crate::export_wasm_guest!(@maybe_export_client_messages $factory, $($client_messages)?);
        $crate::export_wasm_guest!(@maybe_export_server_messages $factory, $($server_messages)?);
        $crate::export_wasm_guest!(@maybe_export_worldgen $factory, $($worldgen)?);
        $crate::export_wasm_guest!(
            @maybe_export_character_controller
            $factory,
            $($character_controller)?
        );
        $crate::export_wasm_guest!(
            @maybe_export_client_control_provider
            $factory,
            $($client_control_provider)?
        );
    };

    (@lifecycle_struct $($hook:ident),*) => {{
        let mut hooks = $crate::__private::LifecycleHooks::default();
        $(hooks.$hook = true;)*
        hooks
    }};
    (@bool true) => { true };
    (@bool false) => { false };
    (@bool) => { false };

    (@maybe_export $factory:path, start_client, start_client $(, $rest:ident)*) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_on_start_client(ptr: u32, len: u32) -> u64 {
            let module = $factory();
            $crate::__private::wasm_guest_start_client(&module, ptr, len)
        }
    };
    (@maybe_export $factory:path, start_client, $_head:ident $(, $rest:ident)*) => {
        $crate::export_wasm_guest!(@maybe_export $factory, start_client $(, $rest)*);
    };
    (@maybe_export $factory:path, start_client,) => {};
    (@maybe_export $factory:path, start_client) => {};

    (@maybe_export $factory:path, start_server, start_server $(, $rest:ident)*) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_on_start_server(ptr: u32, len: u32) -> u64 {
            let module = $factory();
            $crate::__private::wasm_guest_start_server(&module, ptr, len)
        }
    };
    (@maybe_export $factory:path, start_server, $_head:ident $(, $rest:ident)*) => {
        $crate::export_wasm_guest!(@maybe_export $factory, start_server $(, $rest)*);
    };
    (@maybe_export $factory:path, start_server,) => {};
    (@maybe_export $factory:path, start_server) => {};

    (@maybe_export $factory:path, tick_client, tick_client $(, $rest:ident)*) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_on_tick_client(ptr: u32, len: u32) -> u64 {
            let module = $factory();
            $crate::__private::wasm_guest_tick_client(&module, ptr, len)
        }
    };
    (@maybe_export $factory:path, tick_client, $_head:ident $(, $rest:ident)*) => {
        $crate::export_wasm_guest!(@maybe_export $factory, tick_client $(, $rest)*);
    };
    (@maybe_export $factory:path, tick_client,) => {};
    (@maybe_export $factory:path, tick_client) => {};

    (@maybe_export $factory:path, tick_server, tick_server $(, $rest:ident)*) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_on_tick_server(ptr: u32, len: u32) -> u64 {
            let module = $factory();
            $crate::__private::wasm_guest_tick_server(&module, ptr, len)
        }
    };
    (@maybe_export $factory:path, tick_server, $_head:ident $(, $rest:ident)*) => {
        $crate::export_wasm_guest!(@maybe_export $factory, tick_server $(, $rest)*);
    };
    (@maybe_export $factory:path, tick_server,) => {};
    (@maybe_export $factory:path, tick_server) => {};

    (@maybe_export_action $factory:path, true) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_handle_action(ptr: u32, len: u32) -> u64 {
            let module = $factory();
            $crate::__private::wasm_guest_handle_action(&module, ptr, len)
        }
    };
    (@maybe_export_action $factory:path, false) => {};
    (@maybe_export_action $factory:path) => {};

    (@maybe_export_client_messages $factory:path, true) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_on_client_messages(ptr: u32, len: u32) -> u64 {
            let module = $factory();
            $crate::__private::wasm_guest_client_messages(&module, ptr, len)
        }
    };
    (@maybe_export_client_messages $factory:path,) => {};
    (@maybe_export_client_messages $factory:path, false) => {};
    (@maybe_export_client_messages $factory:path) => {};

    (@maybe_export_server_messages $factory:path, true) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_on_server_messages(ptr: u32, len: u32) -> u64 {
            let module = $factory();
            $crate::__private::wasm_guest_server_messages(&module, ptr, len)
        }
    };
    (@maybe_export_server_messages $factory:path,) => {};
    (@maybe_export_server_messages $factory:path, false) => {};
    (@maybe_export_server_messages $factory:path) => {};

    (@maybe_export_worldgen $factory:path, true) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_generate_worldgen(ptr: u32, len: u32) -> u64 {
            let module = $factory();
            $crate::__private::wasm_guest_worldgen(&module, ptr, len)
        }
    };
    (@maybe_export_worldgen $factory:path, false) => {};
    (@maybe_export_worldgen $factory:path,) => {};
    (@maybe_export_worldgen $factory:path) => {};

    (@maybe_export_character_controller $factory:path, true) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_init_character_controller(ptr: u32, len: u32) -> u64 {
            let module = $factory();
            $crate::__private::wasm_guest_character_controller_init(&module, ptr, len)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_step_character_controller(ptr: u32, len: u32) -> u64 {
            let module = $factory();
            $crate::__private::wasm_guest_character_controller_step(&module, ptr, len)
        }
    };
    (@maybe_export_character_controller $factory:path, false) => {};
    (@maybe_export_character_controller $factory:path,) => {};
    (@maybe_export_character_controller $factory:path) => {};

    (@maybe_export_client_control_provider $factory:path, true) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_sample_client_control_provider(ptr: u32, len: u32) -> u64 {
            let module = $factory();
            $crate::__private::wasm_guest_client_control_provider(&module, ptr, len)
        }
    };
    (@maybe_export_client_control_provider $factory:path, false) => {};
    (@maybe_export_client_control_provider $factory:path,) => {};
    (@maybe_export_client_control_provider $factory:path) => {};
}

#[macro_export]
macro_rules! export_native_guest {
    (
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),* $(,)?])?
        $(, actions: $actions:tt)?
        $(, client_messages: $client_messages:tt)?
        $(, server_messages: $server_messages:tt)?
        $(, worldgen: $worldgen:tt)?
        $(, character_controller: $character_controller:tt)?
        $(, client_control_provider: $client_control_provider:tt)?
        $(,)?
    ) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_alloc(size: usize) -> *mut u8 {
            $crate::__private::native_guest_alloc(size)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_dealloc(buffer: $crate::NativeGuestBuffer) {
            $crate::__private::native_guest_dealloc(buffer)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_set_native_runtime_bridge(
            bridge: $crate::NativeRuntimeBridge,
        ) {
            $crate::__private::native_guest_set_runtime_bridge(bridge)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_negotiate(
            input: $crate::NativeGuestInput,
        ) -> $crate::NativeGuestBuffer {
            let module = $factory();
            $crate::__private::assert_export_surface(
                &module,
                $crate::export_native_guest!(@lifecycle_struct $($($lifecycle),*)?),
                $crate::export_native_guest!(@bool $($actions)?),
                $crate::__private::MessageHooks {
                    client: $crate::export_native_guest!(@bool $($client_messages)?),
                    server: $crate::export_native_guest!(@bool $($server_messages)?),
                },
                $crate::ProviderHooks {
                    world: $crate::WorldProviderHooks {
                        worldgen: $crate::export_native_guest!(@bool $($worldgen)?),
                    },
                    avatar: $crate::AvatarProviderHooks {
                        character_controller: $crate::export_native_guest!(@bool $($character_controller)?),
                        client_control_provider: $crate::export_native_guest!(@bool $($client_control_provider)?),
                    },
                },
            );
            $crate::__private::native_guest_negotiate(&module, input)
        }

        $crate::export_native_guest!(@maybe_export $factory, start_client, $($($lifecycle),*)?);
        $crate::export_native_guest!(@maybe_export $factory, start_server, $($($lifecycle),*)?);
        $crate::export_native_guest!(@maybe_export $factory, tick_client, $($($lifecycle),*)?);
        $crate::export_native_guest!(@maybe_export $factory, tick_server, $($($lifecycle),*)?);
        $crate::export_native_guest!(@maybe_export_action $factory, $($actions)?);
        $crate::export_native_guest!(@maybe_export_client_messages $factory, $($client_messages)?);
        $crate::export_native_guest!(@maybe_export_server_messages $factory, $($server_messages)?);
        $crate::export_native_guest!(@maybe_export_worldgen $factory, $($worldgen)?);
        $crate::export_native_guest!(
            @maybe_export_character_controller
            $factory,
            $($character_controller)?
        );
        $crate::export_native_guest!(
            @maybe_export_client_control_provider
            $factory,
            $($client_control_provider)?
        );
    };

    (@lifecycle_struct $($hook:ident),*) => {{
        let mut hooks = $crate::__private::LifecycleHooks::default();
        $(hooks.$hook = true;)*
        hooks
    }};
    (@bool true) => { true };
    (@bool false) => { false };
    (@bool) => { false };

    (@maybe_export $factory:path, start_client, start_client $(, $rest:ident)*) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_on_start_client(
            input: $crate::NativeGuestInput,
        ) -> $crate::NativeGuestBuffer {
            let module = $factory();
            $crate::__private::native_guest_start_client(&module, input)
        }
    };
    (@maybe_export $factory:path, start_client, $_head:ident $(, $rest:ident)*) => {
        $crate::export_native_guest!(@maybe_export $factory, start_client $(, $rest)*);
    };
    (@maybe_export $factory:path, start_client,) => {};
    (@maybe_export $factory:path, start_client) => {};

    (@maybe_export $factory:path, start_server, start_server $(, $rest:ident)*) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_on_start_server(
            input: $crate::NativeGuestInput,
        ) -> $crate::NativeGuestBuffer {
            let module = $factory();
            $crate::__private::native_guest_start_server(&module, input)
        }
    };
    (@maybe_export $factory:path, start_server, $_head:ident $(, $rest:ident)*) => {
        $crate::export_native_guest!(@maybe_export $factory, start_server $(, $rest)*);
    };
    (@maybe_export $factory:path, start_server,) => {};
    (@maybe_export $factory:path, start_server) => {};

    (@maybe_export $factory:path, tick_client, tick_client $(, $rest:ident)*) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_on_tick_client(
            input: $crate::NativeGuestInput,
        ) -> $crate::NativeGuestBuffer {
            let module = $factory();
            $crate::__private::native_guest_tick_client(&module, input)
        }
    };
    (@maybe_export $factory:path, tick_client, $_head:ident $(, $rest:ident)*) => {
        $crate::export_native_guest!(@maybe_export $factory, tick_client $(, $rest)*);
    };
    (@maybe_export $factory:path, tick_client,) => {};
    (@maybe_export $factory:path, tick_client) => {};

    (@maybe_export $factory:path, tick_server, tick_server $(, $rest:ident)*) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_on_tick_server(
            input: $crate::NativeGuestInput,
        ) -> $crate::NativeGuestBuffer {
            let module = $factory();
            $crate::__private::native_guest_tick_server(&module, input)
        }
    };
    (@maybe_export $factory:path, tick_server, $_head:ident $(, $rest:ident)*) => {
        $crate::export_native_guest!(@maybe_export $factory, tick_server $(, $rest)*);
    };
    (@maybe_export $factory:path, tick_server,) => {};
    (@maybe_export $factory:path, tick_server) => {};

    (@maybe_export_action $factory:path, true) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_handle_action(
            input: $crate::NativeGuestInput,
        ) -> $crate::NativeGuestBuffer {
            let module = $factory();
            $crate::__private::native_guest_handle_action(&module, input)
        }
    };
    (@maybe_export_action $factory:path, false) => {};
    (@maybe_export_action $factory:path) => {};

    (@maybe_export_client_messages $factory:path, true) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_on_client_messages(
            input: $crate::NativeGuestInput,
        ) -> $crate::NativeGuestBuffer {
            let module = $factory();
            $crate::__private::native_guest_client_messages(&module, input)
        }
    };
    (@maybe_export_client_messages $factory:path,) => {};
    (@maybe_export_client_messages $factory:path, false) => {};
    (@maybe_export_client_messages $factory:path) => {};

    (@maybe_export_server_messages $factory:path, true) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_on_server_messages(
            input: $crate::NativeGuestInput,
        ) -> $crate::NativeGuestBuffer {
            let module = $factory();
            $crate::__private::native_guest_server_messages(&module, input)
        }
    };
    (@maybe_export_server_messages $factory:path,) => {};
    (@maybe_export_server_messages $factory:path, false) => {};
    (@maybe_export_server_messages $factory:path) => {};

    (@maybe_export_worldgen $factory:path, true) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_generate_worldgen(
            input: $crate::NativeGuestInput,
        ) -> $crate::NativeGuestBuffer {
            let module = $factory();
            $crate::__private::native_guest_worldgen(&module, input)
        }
    };
    (@maybe_export_worldgen $factory:path, false) => {};
    (@maybe_export_worldgen $factory:path,) => {};
    (@maybe_export_worldgen $factory:path) => {};

    (@maybe_export_character_controller $factory:path, true) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_init_character_controller(
            input: $crate::NativeGuestInput,
        ) -> $crate::NativeGuestBuffer {
            let module = $factory();
            $crate::__private::native_guest_character_controller_init(&module, input)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_step_character_controller(
            input: $crate::NativeGuestInput,
        ) -> $crate::NativeGuestBuffer {
            let module = $factory();
            $crate::__private::native_guest_character_controller_step(&module, input)
        }
    };
    (@maybe_export_character_controller $factory:path, false) => {};
    (@maybe_export_character_controller $factory:path,) => {};
    (@maybe_export_character_controller $factory:path) => {};

    (@maybe_export_client_control_provider $factory:path, true) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn freven_guest_sample_client_control_provider(
            input: $crate::NativeGuestInput,
        ) -> $crate::NativeGuestBuffer {
            let module = $factory();
            $crate::__private::native_guest_client_control_provider(&module, input)
        }
    };
    (@maybe_export_client_control_provider $factory:path, false) => {};
    (@maybe_export_client_control_provider $factory:path,) => {};
    (@maybe_export_client_control_provider $factory:path) => {};
}

#[macro_export]
macro_rules! wasm_guest {
    (
        guest_id: $guest_id:expr
        $(, registration: { $($registration:tt)* })?
        $(, lifecycle: { $($lifecycle:ident : $lifecycle_handler:expr),* $(,)? })?
        $(, client_messages: $client_messages_handler:expr)?
        $(, server_messages: $server_messages_handler:expr)?
        $(, actions: {
            $(
                $action_key:expr => {
                    binding_id: $binding_id:expr,
                    handler: $action_handler:expr
                    $(,)?
                }
            ),* $(,)?
        })?
        $(,)?
    ) => {
        #[doc(hidden)]
        fn __freven_guest_sdk_module() -> $crate::GuestModule {
            $crate::wasm_guest!(
                @module
                guest_id: $guest_id
                $(, registration: { $($registration)* })?
                $(, lifecycle: { $($lifecycle : $lifecycle_handler),* })?
                $(, client_messages: $client_messages_handler)?
                $(, server_messages: $server_messages_handler)?
                $(, actions: {
                    $(
                        $action_key => {
                            binding_id: $binding_id,
                            handler: $action_handler,
                        }
                    ),*
                })?
            )
        }

        $crate::wasm_guest!(
            @export
            factory: __freven_guest_sdk_module
            $(, registration: { $($registration)* })?
            $(, lifecycle: [$($lifecycle),*])?
            $(, client_messages: [$client_messages_handler])?
            $(, server_messages: [$server_messages_handler])?
            $(, actions: [$($action_key),*])?
        );
    };

    (
        @module
        guest_id: $guest_id:expr
        $(, registration: { $($registration:tt)* })?
        $(, lifecycle: { $($lifecycle:ident : $lifecycle_handler:expr),* $(,)? })?
        $(, client_messages: $client_messages_handler:expr)?
        $(, server_messages: $server_messages_handler:expr)?
        $(, actions: {
            $(
                $action_key:expr => {
                    binding_id: $binding_id:expr,
                    handler: $action_handler:expr
                    $(,)?
                }
            ),* $(,)?
        })?
        $(,)?
    ) => {{
        let module = $crate::GuestModule::new($guest_id);
        $(
            let module = $crate::wasm_guest!(@registration module, $($registration)*);
        )?
        $(
            $(
                let module = $crate::wasm_guest!(
                    @register_lifecycle
                    module,
                    $lifecycle,
                    $lifecycle_handler
                );
            )*
        )?
        $(
            let module = module.on_client_messages($client_messages_handler);
        )?
        $(
            let module = module.on_server_messages($server_messages_handler);
        )?
        $(
            $(
                let module = module.action($action_key, $binding_id, $action_handler);
            )*
        )?
        module
    }};

    (@register_lifecycle $module:ident, start_client, $handler:expr) => { $module.on_start_client($handler) };
    (@register_lifecycle $module:ident, start_server, $handler:expr) => { $module.on_start_server($handler) };
    (@register_lifecycle $module:ident, tick_client, $handler:expr) => { $module.on_tick_client($handler) };
    (@register_lifecycle $module:ident, tick_server, $handler:expr) => { $module.on_tick_server($handler) };

    (@export
        factory: $factory:path
        , registration: { $($registration:tt)* }
        $(, lifecycle: [$($lifecycle:ident),*])?
        $(, client_messages: [$client_messages:expr])?
        $(, server_messages: [$server_messages:expr])?
        $(, actions: [$($action_key:expr),*])?
    ) => {
        $crate::wasm_guest!(
            @scan_registration_for_exports
            factory: $factory
            , registration: { $($registration)* }
            $(, lifecycle: [$($lifecycle),*])?
            $(, client_messages: [$client_messages])?
            $(, server_messages: [$server_messages])?
            $(, actions: [$($action_key),*])?
            , worldgen: false
            , character_controller: false
            , client_control_provider: false
        );
    };
    (@export
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),*])?
        $(, client_messages: [$client_messages:expr])?
        $(, server_messages: [$server_messages:expr])?
        $(, actions: [$($action_key:expr),*])?
    ) => {
        $crate::wasm_guest!(
            @export_flags
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
            $(, client_messages: [$client_messages])?
            $(, server_messages: [$server_messages])?
            $(, actions: [$($action_key),*])?
            , worldgen: false
            , character_controller: false
            , client_control_provider: false
        );
    };

    (@scan_registration_for_exports
        factory: $factory:path
        , registration: {}
        $(, lifecycle: [$($lifecycle:ident),*])?
        $(, client_messages: [$client_messages:expr])?
        $(, server_messages: [$server_messages:expr])?
        $(, actions: [$($action_key:expr),*])?
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::wasm_guest!(
            @export_flags
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
            $(, client_messages: [$client_messages])?
            $(, server_messages: [$server_messages])?
            $(, actions: [$($action_key),*])?
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@scan_registration_for_exports
        factory: $factory:path
        , registration: { , $($rest:tt)* }
        $(, lifecycle: [$($lifecycle:ident),*])?
        $(, client_messages: [$client_messages:expr])?
        $(, server_messages: [$server_messages:expr])?
        $(, actions: [$($action_key:expr),*])?
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::wasm_guest!(
            @scan_registration_for_exports
            factory: $factory
            , registration: { $($rest)* }
            $(, lifecycle: [$($lifecycle),*])?
            $(, client_messages: [$client_messages])?
            $(, server_messages: [$server_messages])?
            $(, actions: [$($action_key),*])?
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@scan_registration_for_exports
        factory: $factory:path
        , registration: { block: $key:expr => $def:expr $(, $($rest:tt)*)? }
        $(, lifecycle: [$($lifecycle:ident),*])?
        $(, client_messages: [$client_messages:expr])?
        $(, server_messages: [$server_messages:expr])?
        $(, actions: [$($action_key:expr),*])?
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::wasm_guest!(
            @scan_registration_for_exports
            factory: $factory
            , registration: { $($($rest)*)? }
            $(, lifecycle: [$($lifecycle),*])?
            $(, client_messages: [$client_messages])?
            $(, server_messages: [$server_messages])?
            $(, actions: [$($action_key),*])?
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@scan_registration_for_exports
        factory: $factory:path
        , registration: { component: $key:expr => $codec:expr $(, $($rest:tt)*)? }
        $(, lifecycle: [$($lifecycle:ident),*])?
        $(, client_messages: [$client_messages:expr])?
        $(, server_messages: [$server_messages:expr])?
        $(, actions: [$($action_key:expr),*])?
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::wasm_guest!(
            @scan_registration_for_exports
            factory: $factory
            , registration: { $($($rest)*)? }
            $(, lifecycle: [$($lifecycle),*])?
            $(, client_messages: [$client_messages])?
            $(, server_messages: [$server_messages])?
            $(, actions: [$($action_key),*])?
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@scan_registration_for_exports
        factory: $factory:path
        , registration: { message: $key:expr => $codec:expr $(, $($rest:tt)*)? }
        $(, lifecycle: [$($lifecycle:ident),*])?
        $(, client_messages: [$client_messages:expr])?
        $(, server_messages: [$server_messages:expr])?
        $(, actions: [$($action_key:expr),*])?
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::wasm_guest!(
            @scan_registration_for_exports
            factory: $factory
            , registration: { $($($rest)*)? }
            $(, lifecycle: [$($lifecycle),*])?
            $(, client_messages: [$client_messages])?
            $(, server_messages: [$server_messages])?
            $(, actions: [$($action_key),*])?
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@scan_registration_for_exports
        factory: $factory:path
        , registration: { worldgen: $key:expr => $handler:expr $(, $($rest:tt)*)? }
        $(, lifecycle: [$($lifecycle:ident),*])?
        $(, client_messages: [$client_messages:expr])?
        $(, server_messages: [$server_messages:expr])?
        $(, actions: [$($action_key:expr),*])?
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::wasm_guest!(
            @scan_registration_for_exports
            factory: $factory
            , registration: { $($($rest)*)? }
            $(, lifecycle: [$($lifecycle),*])?
            $(, client_messages: [$client_messages])?
            $(, server_messages: [$server_messages])?
            $(, actions: [$($action_key),*])?
            , worldgen: true
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@scan_registration_for_exports
        factory: $factory:path
        , registration: { worldgen: $key:expr $(, $($rest:tt)*)? }
        $(, lifecycle: [$($lifecycle:ident),*])?
        $(, client_messages: [$client_messages:expr])?
        $(, server_messages: [$server_messages:expr])?
        $(, actions: [$($action_key:expr),*])?
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::wasm_guest!(
            @scan_registration_for_exports
            factory: $factory
            , registration: { $($($rest)*)? }
            $(, lifecycle: [$($lifecycle),*])?
            $(, client_messages: [$client_messages])?
            $(, server_messages: [$server_messages])?
            $(, actions: [$($action_key),*])?
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@scan_registration_for_exports
        factory: $factory:path
        , registration: { character_controller: $key:expr => { $($handler:tt)* } $(, $($rest:tt)*)? }
        $(, lifecycle: [$($lifecycle:ident),*])?
        $(, client_messages: [$client_messages:expr])?
        $(, server_messages: [$server_messages:expr])?
        $(, actions: [$($action_key:expr),*])?
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::wasm_guest!(
            @scan_registration_for_exports
            factory: $factory
            , registration: { $($($rest)*)? }
            $(, lifecycle: [$($lifecycle),*])?
            $(, client_messages: [$client_messages])?
            $(, server_messages: [$server_messages])?
            $(, actions: [$($action_key),*])?
            , worldgen: $worldgen
            , character_controller: true
            , client_control_provider: $client_control_provider
        );
    };
    (@scan_registration_for_exports
        factory: $factory:path
        , registration: { character_controller: $key:expr $(, $($rest:tt)*)? }
        $(, lifecycle: [$($lifecycle:ident),*])?
        $(, client_messages: [$client_messages:expr])?
        $(, server_messages: [$server_messages:expr])?
        $(, actions: [$($action_key:expr),*])?
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::wasm_guest!(
            @scan_registration_for_exports
            factory: $factory
            , registration: { $($($rest)*)? }
            $(, lifecycle: [$($lifecycle),*])?
            $(, client_messages: [$client_messages])?
            $(, server_messages: [$server_messages])?
            $(, actions: [$($action_key),*])?
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@scan_registration_for_exports
        factory: $factory:path
        , registration: { client_control_provider: $key:expr => $handler:expr $(, $($rest:tt)*)? }
        $(, lifecycle: [$($lifecycle:ident),*])?
        $(, client_messages: [$client_messages:expr])?
        $(, server_messages: [$server_messages:expr])?
        $(, actions: [$($action_key:expr),*])?
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::wasm_guest!(
            @scan_registration_for_exports
            factory: $factory
            , registration: { $($($rest)*)? }
            $(, lifecycle: [$($lifecycle),*])?
            $(, client_messages: [$client_messages])?
            $(, server_messages: [$server_messages])?
            $(, actions: [$($action_key),*])?
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: true
        );
    };
    (@scan_registration_for_exports
        factory: $factory:path
        , registration: { client_control_provider: $key:expr $(, $($rest:tt)*)? }
        $(, lifecycle: [$($lifecycle:ident),*])?
        $(, client_messages: [$client_messages:expr])?
        $(, server_messages: [$server_messages:expr])?
        $(, actions: [$($action_key:expr),*])?
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::wasm_guest!(
            @scan_registration_for_exports
            factory: $factory
            , registration: { $($($rest)*)? }
            $(, lifecycle: [$($lifecycle),*])?
            $(, client_messages: [$client_messages])?
            $(, server_messages: [$server_messages])?
            $(, actions: [$($action_key),*])?
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@scan_registration_for_exports
        factory: $factory:path
        , registration: { channel: $key:expr => $config:expr $(, $($rest:tt)*)? }
        $(, lifecycle: [$($lifecycle:ident),*])?
        $(, client_messages: [$client_messages:expr])?
        $(, server_messages: [$server_messages:expr])?
        $(, actions: [$($action_key:expr),*])?
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::wasm_guest!(
            @scan_registration_for_exports
            factory: $factory
            , registration: { $($($rest)*)? }
            $(, lifecycle: [$($lifecycle),*])?
            $(, client_messages: [$client_messages])?
            $(, server_messages: [$server_messages])?
            $(, actions: [$($action_key),*])?
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@scan_registration_for_exports
        factory: $factory:path
        , registration: { capability: $key:expr $(, $($rest:tt)*)? }
        $(, lifecycle: [$($lifecycle:ident),*])?
        $(, client_messages: [$client_messages:expr])?
        $(, server_messages: [$server_messages:expr])?
        $(, actions: [$($action_key:expr),*])?
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::wasm_guest!(
            @scan_registration_for_exports
            factory: $factory
            , registration: { $($($rest)*)? }
            $(, lifecycle: [$($lifecycle),*])?
            $(, client_messages: [$client_messages])?
            $(, server_messages: [$server_messages])?
            $(, actions: [$($action_key),*])?
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };

    (@export_flags
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),*])?
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::export_wasm_guest!(
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
            , actions: false
            , client_messages: false
            , server_messages: false
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@export_flags
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),*])?
        , actions: []
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::wasm_guest!(
            @export_flags
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@export_flags
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),*])?
        , actions: [$first:expr $(, $rest:expr)*]
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::export_wasm_guest!(
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
            , actions: true
            , client_messages: false
            , server_messages: false
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@export_flags
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),*])?
        , client_messages: [$handler:expr]
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::export_wasm_guest!(
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
            , actions: false
            , client_messages: true
            , server_messages: false
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@export_flags
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),*])?
        , client_messages: [$handler:expr]
        , actions: []
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::wasm_guest!(
            @export_flags
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
            , client_messages: [$handler]
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@export_flags
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),*])?
        , client_messages: [$handler:expr]
        , actions: [$first:expr $(, $rest:expr)*]
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::export_wasm_guest!(
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
            , actions: true
            , client_messages: true
            , server_messages: false
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@export_flags
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),*])?
        , server_messages: [$handler:expr]
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::export_wasm_guest!(
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
            , actions: false
            , client_messages: false
            , server_messages: true
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@export_flags
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),*])?
        , server_messages: [$handler:expr]
        , actions: []
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::wasm_guest!(
            @export_flags
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
            , server_messages: [$handler]
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@export_flags
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),*])?
        , server_messages: [$handler:expr]
        , actions: [$first:expr $(, $rest:expr)*]
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::export_wasm_guest!(
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
            , actions: true
            , client_messages: false
            , server_messages: true
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@export_flags
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),*])?
        , client_messages: [$client_handler:expr]
        , server_messages: [$server_handler:expr]
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::export_wasm_guest!(
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
            , actions: false
            , client_messages: true
            , server_messages: true
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@export_flags
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),*])?
        , client_messages: [$client_handler:expr]
        , server_messages: [$server_handler:expr]
        , actions: []
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::wasm_guest!(
            @export_flags
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
            , client_messages: [$client_handler]
            , server_messages: [$server_handler]
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };
    (@export_flags
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),*])?
        , client_messages: [$client_handler:expr]
        , server_messages: [$server_handler:expr]
        , actions: [$first:expr $(, $rest:expr)*]
        , worldgen: $worldgen:tt
        , character_controller: $character_controller:tt
        , client_control_provider: $client_control_provider:tt
    ) => {
        $crate::export_wasm_guest!(
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
            , actions: true
            , client_messages: true
            , server_messages: true
            , worldgen: $worldgen
            , character_controller: $character_controller
            , client_control_provider: $client_control_provider
        );
    };

    (@registration $module:expr) => { $module };
    (@registration $module:expr,) => { $module };
    (@registration $module:expr, block: $key:expr => $def:expr $(, $($rest:tt)*)?) => {
        $crate::wasm_guest!(@registration $module.register_block($key, $def) $(, $($rest)*)?)
    };
    (@registration $module:expr, component: $key:expr => $codec:expr $(, $($rest:tt)*)?) => {
        $crate::wasm_guest!(@registration $module.register_component($key, $codec) $(, $($rest)*)?)
    };
    (@registration $module:expr, message: $key:expr => $codec:expr $(, $($rest:tt)*)?) => {
        $crate::wasm_guest!(@registration $module.register_message($key, $codec) $(, $($rest)*)?)
    };
    (@registration $module:expr, worldgen: $key:expr => $handler:expr $(, $($rest:tt)*)?) => {
        $crate::wasm_guest!(@registration $module.register_worldgen_handler($key, $handler) $(, $($rest)*)?)
    };
    (@registration $module:expr, worldgen: $key:expr $(, $($rest:tt)*)?) => {
        compile_error!(
            "wasm_guest! worldgen registrations require a handler: worldgen: \"mod:key\" => handler"
        )
    };
    (@registration $module:expr, character_controller: $key:expr => { init: $init:expr, step: $step:expr $(,)? } $(, $($rest:tt)*)?) => {
        $crate::wasm_guest!(@registration $module.register_character_controller_handler($key, $init, $step) $(, $($rest)*)?)
    };
    (@registration $module:expr, character_controller: $key:expr => { step: $step:expr, init: $init:expr $(,)? } $(, $($rest:tt)*)?) => {
        $crate::wasm_guest!(@registration $module.register_character_controller_handler($key, $init, $step) $(, $($rest)*)?)
    };
    (@registration $module:expr, character_controller: $key:expr $(, $($rest:tt)*)?) => {
        compile_error!(
            "wasm_guest! character_controller registrations require init/step handlers: character_controller: \"mod:key\" => { init: ..., step: ... }"
        )
    };
    (@registration $module:expr, client_control_provider: $key:expr => $handler:expr $(, $($rest:tt)*)?) => {
        $crate::wasm_guest!(@registration $module.register_client_control_provider_handler($key, $handler) $(, $($rest)*)?)
    };
    (@registration $module:expr, client_control_provider: $key:expr $(, $($rest:tt)*)?) => {
        compile_error!(
            "wasm_guest! client_control_provider registrations require a handler: client_control_provider: \"mod:key\" => handler"
        )
    };
    (@registration $module:expr, channel: $key:expr => $config:expr $(, $($rest:tt)*)?) => {
        $crate::wasm_guest!(@registration $module.register_channel($key, $config) $(, $($rest)*)?)
    };
    (@registration $module:expr, capability: $key:expr $(, $($rest:tt)*)?) => {
        $crate::wasm_guest!(@registration $module.declare_capability($key) $(, $($rest)*)?)
    };

}

#[macro_export]
macro_rules! stateful_wasm_guest {
    (
        guest_id: $guest_id:expr,
        session_state: $state_ty:ty = $session_factory:expr
        $(, registration: { $($registration:tt)* })?
        $(, lifecycle: { $($lifecycle:ident : $lifecycle_handler:expr),* $(,)? })?
        $(, client_messages: $client_messages_handler:expr)?
        $(, server_messages: $server_messages_handler:expr)?
        $(, actions: {
            $(
                $action_key:expr => {
                    binding_id: $binding_id:expr,
                    handler: $action_handler:expr
                    $(,)?
                }
            ),* $(,)?
        })?
        $(,)?
    ) => {
        std::thread_local! {
            static __FREVEN_GUEST_SDK_SESSION_STORE: core::cell::RefCell<
                $crate::StatefulGuestSessionStore<$state_ty>
            > = const {
                core::cell::RefCell::new($crate::StatefulGuestSessionStore::new())
            };
        }

        #[doc(hidden)]
        fn __freven_guest_sdk_stateful_module() -> $crate::StatefulGuestModule<$state_ty> {
            $crate::stateful_wasm_guest!(
                @module
                guest_id: $guest_id,
                session_factory: $session_factory,
                session_store: &__FREVEN_GUEST_SDK_SESSION_STORE
                $(, registration: { $($registration)* })?
                $(, lifecycle: { $($lifecycle : $lifecycle_handler),* })?
                $(, client_messages: $client_messages_handler)?
                $(, server_messages: $server_messages_handler)?
                $(, actions: {
                    $(
                        $action_key => {
                            binding_id: $binding_id,
                            handler: $action_handler,
                        }
                    ),*
                })?
            )
        }

        $crate::wasm_guest!(
            @export
            factory: __freven_guest_sdk_stateful_module
            $(, registration: { $($registration)* })?
            $(, lifecycle: [$($lifecycle),*])?
            $(, client_messages: [$client_messages_handler])?
            $(, server_messages: [$server_messages_handler])?
            $(, actions: [$($action_key),*])?
        );
    };

    (
        @module
        guest_id: $guest_id:expr,
        session_factory: $session_factory:expr,
        session_store: $session_store:expr
        $(, registration: { $($registration:tt)* })?
        $(, lifecycle: { $($lifecycle:ident : $lifecycle_handler:expr),* $(,)? })?
        $(, client_messages: $client_messages_handler:expr)?
        $(, server_messages: $server_messages_handler:expr)?
        $(, actions: {
            $(
                $action_key:expr => {
                    binding_id: $binding_id:expr,
                    handler: $action_handler:expr
                    $(,)?
                }
            ),* $(,)?
        })?
        $(,)?
    ) => {{
        let module = $crate::StatefulGuestModule::new($guest_id, $session_factory, $session_store);
        $(
            let module = $crate::stateful_wasm_guest!(@registration module, $($registration)*);
        )?
        $(
            $(
                let module = $crate::stateful_wasm_guest!(
                    @register_lifecycle
                    module,
                    $lifecycle,
                    $lifecycle_handler
                );
            )*
        )?
        $(
            let module = module.on_client_messages($client_messages_handler);
        )?
        $(
            let module = module.on_server_messages($server_messages_handler);
        )?
        $(
            $(
                let module = module.action($action_key, $binding_id, $action_handler);
            )*
        )?
        module
    }};

    (@register_lifecycle $module:ident, start_client, $handler:expr) => { $module.on_start_client($handler) };
    (@register_lifecycle $module:ident, start_server, $handler:expr) => { $module.on_start_server($handler) };
    (@register_lifecycle $module:ident, tick_client, $handler:expr) => { $module.on_tick_client($handler) };
    (@register_lifecycle $module:ident, tick_server, $handler:expr) => { $module.on_tick_server($handler) };

    (@registration $module:expr) => { $module };
    (@registration $module:expr,) => { $module };
    (@registration $module:expr, block: $key:expr => $def:expr $(, $($rest:tt)*)?) => {
        $crate::stateful_wasm_guest!(@registration $module.register_block($key, $def) $(, $($rest)*)?)
    };
    (@registration $module:expr, component: $key:expr => $codec:expr $(, $($rest:tt)*)?) => {
        $crate::stateful_wasm_guest!(@registration $module.register_component($key, $codec) $(, $($rest)*)?)
    };
    (@registration $module:expr, message: $key:expr => $codec:expr $(, $($rest:tt)*)?) => {
        $crate::stateful_wasm_guest!(@registration $module.register_message($key, $codec) $(, $($rest)*)?)
    };
    (@registration $module:expr, worldgen: $key:expr => $handler:expr $(, $($rest:tt)*)?) => {
        $crate::stateful_wasm_guest!(@registration $module.register_worldgen_handler($key, $handler) $(, $($rest)*)?)
    };
    (@registration $module:expr, worldgen: $key:expr $(, $($rest:tt)*)?) => {
        compile_error!(
            "stateful_wasm_guest! worldgen registrations require a handler: worldgen: \"mod:key\" => handler"
        )
    };
    (@registration $module:expr, character_controller: $key:expr => { init: $init:expr, step: $step:expr $(,)? } $(, $($rest:tt)*)?) => {
        $crate::stateful_wasm_guest!(@registration $module.register_character_controller_handler($key, $init, $step) $(, $($rest)*)?)
    };
    (@registration $module:expr, character_controller: $key:expr => { step: $step:expr, init: $init:expr $(,)? } $(, $($rest:tt)*)?) => {
        $crate::stateful_wasm_guest!(@registration $module.register_character_controller_handler($key, $init, $step) $(, $($rest)*)?)
    };
    (@registration $module:expr, character_controller: $key:expr $(, $($rest:tt)*)?) => {
        compile_error!(
            "stateful_wasm_guest! character_controller registrations require init/step handlers: character_controller: \"mod:key\" => { init: ..., step: ... }"
        )
    };
    (@registration $module:expr, client_control_provider: $key:expr => $handler:expr $(, $($rest:tt)*)?) => {
        $crate::stateful_wasm_guest!(@registration $module.register_client_control_provider_handler($key, $handler) $(, $($rest)*)?)
    };
    (@registration $module:expr, client_control_provider: $key:expr $(, $($rest:tt)*)?) => {
        compile_error!(
            "stateful_wasm_guest! client_control_provider registrations require a handler: client_control_provider: \"mod:key\" => handler"
        )
    };
    (@registration $module:expr, channel: $key:expr => $config:expr $(, $($rest:tt)*)?) => {
        $crate::stateful_wasm_guest!(@registration $module.register_channel($key, $config) $(, $($rest)*)?)
    };
    (@registration $module:expr, capability: $key:expr $(, $($rest:tt)*)?) => {
        $crate::stateful_wasm_guest!(@registration $module.declare_capability($key) $(, $($rest)*)?)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use freven_guest::{ChannelConfig, ChannelDirection, ChannelOrdering, ChannelReliability};

    fn message_channel() -> ChannelConfig {
        ChannelConfig {
            reliability: ChannelReliability::Reliable,
            ordering: ChannelOrdering::Ordered,
            direction: ChannelDirection::Bidirectional,
            budget: None,
        }
    }

    fn module() -> GuestModule {
        GuestModule::new("freven.test.guest")
            .register_block(
                "freven.test:stone",
                BlockDescriptor::new(true, true, RenderLayer::Opaque, 0, 7),
            )
            .register_component("freven.test:name", ComponentCodec::RawBytes)
            .register_message("freven.test:echo", MessageCodec::RawBytes)
            .register_worldgen("freven.test:flat")
            .register_character_controller("freven.test:humanoid")
            .register_client_control_provider("freven.test:controls")
            .register_channel("freven.test:echo", message_channel())
            .declare_capability("max_call_millis")
            .on_start_server(|_| LifecycleResponse::default().finish())
            .on_tick_server(|_| LifecycleResponse::default().finish())
            .on_server_messages(|ctx| {
                let Some(msg) = ctx.messages().first() else {
                    return ServerMessageResponse::default();
                };
                ServerMessageResponse::default().send_to(ServerOutboundMessage {
                    player_id: msg.player_id,
                    scope: msg.scope,
                    channel_id: msg.channel_id,
                    message_id: msg.message_id,
                    seq: msg.seq,
                    payload: msg.payload.clone(),
                })
            })
            .action("freven.test:place_block", 7, |_| {
                ActionResponse::applied()
                    .set_block((1, 2, 3), BlockRuntimeId(9))
                    .finish()
            })
    }

    fn provider_worldgen(_: WorldGenContext<'_>) -> WorldGenCallResult {
        WorldGenCallResult {
            output: WorldGenOutput {
                writes: vec![WorldTerrainWrite::FillSection {
                    sy: 0.into(),
                    block_id: BlockRuntimeId(7),
                }],
                bootstrap: WorldGenBootstrapOutput {
                    initial_world_spawn_hint: Some(InitialWorldSpawnHint {
                        feet_position: [0.5, 65.0, 0.5],
                    }),
                },
            },
        }
    }

    fn provider_character_init(
        _: CharacterControllerInitContext<'_>,
    ) -> CharacterControllerInitResult {
        CharacterControllerInitResult {
            config: CharacterConfig {
                shape: CharacterShape::Aabb {
                    half_extents: [0.4, 0.9, 0.4],
                },
                max_speed_ground: 5.0,
                max_speed_air: 3.0,
                accel_ground: 12.0,
                accel_air: 4.0,
                gravity: 9.8,
                jump_impulse: 5.5,
                step_height: 0.25,
                skin_width: 0.001,
            },
        }
    }

    fn provider_character_step(
        ctx: CharacterControllerStepContext<'_>,
    ) -> CharacterControllerStepResult {
        let mut state = ctx.state();
        state.pos[1] += 1.0;
        CharacterControllerStepResult { state }
    }

    fn provider_client_control(_: ClientControlProviderContext<'_>) -> ClientControlSampleResult {
        ClientControlSampleResult {
            output: ClientControlOutput {
                input: vec![1, 2, 3],
                view_yaw_deg_mdeg: 12_000,
                view_pitch_deg_mdeg: -3_000,
            },
        }
    }

    #[test]
    #[should_panic(expected = "freven_guest_sdk guest_id must not be empty")]
    fn empty_guest_id_panics() {
        let _ = GuestModule::new("");
    }

    #[test]
    fn description_reflects_registered_families() {
        let description = module().description();
        assert_eq!(description.guest_id, "freven.test.guest");
        assert_eq!(description.registration.blocks.len(), 1);
        assert_eq!(description.registration.components.len(), 1);
        assert_eq!(description.registration.messages.len(), 1);
        assert_eq!(description.registration.world.worldgen.len(), 1);
        assert_eq!(
            description.registration.avatar.character_controllers.len(),
            1
        );
        assert_eq!(
            description
                .registration
                .avatar
                .client_control_providers
                .len(),
            1
        );
        assert_eq!(description.registration.channels.len(), 1);
        assert_eq!(description.registration.actions.len(), 1);
        assert_eq!(description.registration.capabilities.len(), 1);
        assert!(description.callbacks.lifecycle.start_server);
        assert!(description.callbacks.lifecycle.tick_server);
        assert!(description.callbacks.action);
        assert!(description.callbacks.messages.server);
    }

    #[test]
    fn start_input_ext_decodes_toml_config() {
        #[derive(serde::Deserialize)]
        struct TestConfig {
            motd: String,
        }

        let input = StartInput {
            session: RuntimeSessionInfo {
                id: 7,
                side: RuntimeSessionSide::Server,
            },
            experience_id: "freven.test".to_string(),
            mod_id: "freven.test.guest".to_string(),
            config: ModConfigDocument {
                format: ModConfigFormat::Toml,
                text: "motd = \"hello\"".to_string(),
            },
        };

        let decoded: TestConfig = input
            .config_typed()
            .expect("config_typed should decode TOML");
        assert_eq!(input.config_text(), "motd = \"hello\"");
        assert_eq!(decoded.motd, "hello");
    }

    #[test]
    fn stateful_guest_module_rotates_state_by_runtime_session() {
        #[derive(Debug, Default, PartialEq, Eq)]
        struct TestState {
            starts: u32,
            actions: u32,
        }

        std::thread_local! {
            static STORE: RefCell<StatefulGuestSessionStore<TestState>> =
                const { RefCell::new(StatefulGuestSessionStore::new()) };
        }

        let module =
            StatefulGuestModule::new("freven.test.stateful", |_| TestState::default(), &STORE)
                .on_start_server(|state, ctx| {
                    assert_eq!(ctx.session().side, RuntimeSessionSide::Server);
                    state.starts += 1;
                    LifecycleResponse::default().finish()
                })
                .action("freven.test:ping", 3, |state, _| {
                    state.actions += 1;
                    ActionResponse::applied().finish()
                });

        let start = StartInput {
            session: RuntimeSessionInfo {
                id: 11,
                side: RuntimeSessionSide::Server,
            },
            experience_id: "freven.test".to_string(),
            mod_id: "freven.test.stateful".to_string(),
            config: ModConfigDocument::default(),
        };
        let action = ActionInput {
            binding_id: 3,
            player_id: 1,
            level_id: 2,
            stream_epoch: 4,
            action_seq: 8,
            at_input_seq: 16,
            payload: &[],
        };

        let _ = module.handle_start_server(&start);
        let _ = module.handle_action(action.clone());
        let _ = module.handle_action(action.clone());
        STORE.with(|store| {
            let state = store.borrow();
            let current = state.current.as_ref().expect("session should exist");
            assert_eq!(current.info.id, 11);
            assert_eq!(
                current.state,
                TestState {
                    starts: 1,
                    actions: 2
                }
            );
        });

        let next_start = StartInput {
            session: RuntimeSessionInfo {
                id: 12,
                side: RuntimeSessionSide::Server,
            },
            ..start
        };
        let _ = module.handle_start_server(&next_start);
        STORE.with(|store| {
            let state = store.borrow();
            let current = state.current.as_ref().expect("new session should exist");
            assert_eq!(current.info.id, 12);
            assert_eq!(
                current.state,
                TestState {
                    starts: 1,
                    actions: 0
                }
            );
        });
    }

    #[test]
    fn missing_binding_rejects_without_effects() {
        let result = module().handle_action(ActionInput {
            binding_id: 99,
            player_id: 1,
            level_id: 2,
            stream_epoch: 3,
            action_seq: 4,
            at_input_seq: 5,
            payload: &[],
        });

        assert_eq!(result.outcome, ActionOutcome::Rejected);
        assert!(result.output.is_empty());
    }

    #[test]
    fn server_message_handler_round_trips() {
        let result = module().handle_server_messages(ServerMessageInput {
            tick: 5,
            dt_millis: 16,
            messages: vec![ServerInboundMessage {
                player_id: 42,
                scope: MessageScope::Global,
                channel_id: 1,
                message_id: 2,
                seq: Some(3),
                payload: b"hello".to_vec(),
            }],
        });
        assert_eq!(result.output.messages.server.len(), 1);
        assert_eq!(result.output.messages.server[0].payload, b"hello");
    }

    #[test]
    #[should_panic(
        expected = "freven_guest_sdk capability key 'freven.test:dup' was registered more than once"
    )]
    fn duplicate_capability_keys_panic() {
        let _ = GuestModule::new("freven.test.guest")
            .declare_capability("freven.test:dup")
            .declare_capability("freven.test:dup");
    }

    #[test]
    #[should_panic(
        expected = "freven_guest_sdk action key 'freven.test:dup' was registered more than once"
    )]
    fn duplicate_action_keys_panic() {
        let _ = GuestModule::new("freven.test.guest")
            .action("freven.test:dup", 7, |_| ActionResponse::applied().finish())
            .action("freven.test:dup", 8, |_| ActionResponse::applied().finish());
    }

    #[test]
    #[should_panic(expected = "freven_guest_sdk action key must not be empty")]
    fn empty_action_keys_panic() {
        let _ = GuestModule::new("freven.test.guest")
            .action("", 7, |_| ActionResponse::applied().finish());
    }

    #[test]
    #[should_panic(expected = "freven_guest_sdk action key must not be empty")]
    fn whitespace_only_action_keys_panic() {
        let _ = GuestModule::new("freven.test.guest")
            .action("   ", 7, |_| ActionResponse::applied().finish());
    }

    #[test]
    #[should_panic(expected = "freven_guest_sdk binding id 7 was registered more than once")]
    fn duplicate_action_binding_ids_panic() {
        let _ = GuestModule::new("freven.test.guest")
            .action("freven.test:a", 7, |_| ActionResponse::applied().finish())
            .action("freven.test:b", 7, |_| ActionResponse::applied().finish());
    }

    #[test]
    fn callbacks_action_tracks_declared_actions_only() {
        let no_actions = GuestModule::new("freven.test.guest");
        assert!(!no_actions.callbacks().action);

        let with_action = GuestModule::new("freven.test.guest")
            .action("freven.test:a", 1, |_| ActionResponse::rejected().finish());
        assert!(with_action.callbacks().action);
    }

    #[test]
    fn rejected_action_response_finishes_without_effects() {
        let result = ActionResponse::rejected().finish();
        assert_eq!(result.outcome, ActionOutcome::Rejected);
        assert!(result.output.is_empty());
    }

    #[test]
    #[should_panic(expected = "guest input must not be empty")]
    fn action_input_must_not_be_empty() {
        let module = module();
        let _ = __private::wasm_guest_handle_action(&module, 0, 0);
    }

    #[test]
    fn export_surface_assertion_happy_path() {
        let module = module();
        __private::assert_export_surface(
            &module,
            LifecycleHooks {
                start_server: true,
                tick_server: true,
                ..Default::default()
            },
            true,
            MessageHooks {
                client: false,
                server: true,
            },
            ProviderHooks::default(),
        );
    }

    #[test]
    #[should_panic(expected = "freven_guest_sdk export lifecycle does not match")]
    fn export_surface_assertion_rejects_lifecycle_mismatch() {
        __private::assert_export_surface(
            &module(),
            LifecycleHooks::default(),
            true,
            MessageHooks {
                client: false,
                server: true,
            },
            ProviderHooks::default(),
        );
    }

    #[test]
    #[should_panic(expected = "freven_guest_sdk action export does not match")]
    fn export_surface_assertion_rejects_action_mismatch() {
        __private::assert_export_surface(
            &module(),
            LifecycleHooks {
                start_server: true,
                tick_server: true,
                ..Default::default()
            },
            false,
            MessageHooks {
                client: false,
                server: true,
            },
            ProviderHooks::default(),
        );
    }

    #[test]
    #[should_panic(expected = "freven_guest_sdk message export surface does not match")]
    fn export_surface_assertion_rejects_server_message_mismatch() {
        __private::assert_export_surface(
            &module(),
            LifecycleHooks {
                start_server: true,
                tick_server: true,
                ..Default::default()
            },
            true,
            MessageHooks {
                client: false,
                server: false,
            },
            ProviderHooks::default(),
        );
    }

    #[test]
    fn wasm_guest_macro_keeps_surface_and_registration_coherent() {
        let module = wasm_guest!(
            @module
            guest_id: "freven.test.macro"
            , registration: {
                message: "freven.test:macro_message" => MessageCodec::RawBytes,
                channel: "freven.test:macro_channel" => message_channel(),
                capability: "max_call_millis"
            }
            , lifecycle: {
                start_server: |_| LifecycleResponse::default().finish(),
                tick_server: |_| LifecycleResponse::default().finish()
            }
            , server_messages: |_| ServerMessageResponse::default()
            , actions: {
                "freven.test:macro_action" => {
                    binding_id: 17,
                    handler: |_| ActionResponse::applied().finish(),
                }
            }
        );

        let description = module.description();
        assert_eq!(description.guest_id, "freven.test.macro");
        assert_eq!(
            description.registration.messages[0].key,
            "freven.test:macro_message"
        );
        assert_eq!(
            description.registration.channels[0].key,
            "freven.test:macro_channel"
        );
        assert_eq!(
            description.registration.actions[0],
            ActionDeclaration {
                key: "freven.test:macro_action".to_string(),
                binding_id: 17,
            }
        );
        assert_eq!(
            description.registration.capabilities[0].key,
            "max_call_millis"
        );
        assert_eq!(
            description.callbacks.lifecycle,
            LifecycleHooks {
                start_server: true,
                tick_server: true,
                ..Default::default()
            }
        );
        assert!(description.callbacks.action);
        assert!(description.callbacks.messages.server);
    }

    #[test]
    fn wasm_guest_macro_supports_provider_family_handlers() {
        let module = wasm_guest!(
            @module
            guest_id: "freven.test.providers"
            , registration: {
                worldgen: "freven.test:flat" => provider_worldgen,
                character_controller: "freven.test:humanoid" => {
                    init: provider_character_init,
                    step: provider_character_step,
                },
                client_control_provider: "freven.test:controls" => provider_client_control
            }
        );

        let description = module.description();
        assert_eq!(description.registration.world.worldgen.len(), 1);
        assert_eq!(
            description.registration.avatar.character_controllers.len(),
            1
        );
        assert_eq!(
            description
                .registration
                .avatar
                .client_control_providers
                .len(),
            1
        );
        assert_eq!(
            description.callbacks.providers,
            ProviderHooks {
                world: WorldProviderHooks { worldgen: true },
                avatar: AvatarProviderHooks {
                    character_controller: true,
                    client_control_provider: true,
                },
            }
        );

        let worldgen = module.handle_worldgen(WorldGenCallInput {
            key: "freven.test:flat".to_string(),
            init: WorldGenInit::default(),
            request: WorldGenRequest::default(),
        });
        assert_eq!(worldgen.output.writes.len(), 1);
        match &worldgen.output.writes[0] {
            WorldTerrainWrite::FillSection { sy, block_id } => {
                assert_eq!(*sy, 0.into());
                assert_eq!(*block_id, BlockRuntimeId(7));
            }
            write => panic!("unexpected worldgen write: {write:?}"),
        }
        assert_eq!(
            worldgen.output.bootstrap.initial_world_spawn_hint,
            Some(InitialWorldSpawnHint {
                feet_position: [0.5, 65.0, 0.5],
            })
        );

        let init = module.handle_character_controller_init(CharacterControllerInitInput {
            key: "freven.test:humanoid".to_string(),
        });
        assert_eq!(init.config.step_height, 0.25);

        let stepped = module.handle_character_controller_step(CharacterControllerStepInput {
            key: "freven.test:humanoid".to_string(),
            state: CharacterState {
                pos: [0.0, 0.0, 0.0],
                vel: [0.0, 0.0, 0.0],
                on_ground: true,
            },
            input: CharacterControllerInput {
                input: Vec::new(),
                view_yaw_deg_mdeg: 0,
                view_pitch_deg_mdeg: 0,
                timeline: InputTimeline::default(),
            },
            dt_millis: 16,
        });
        assert_eq!(stepped.state.pos[1], 1.0);

        let controls = module.handle_client_control_provider(ClientControlSampleInput {
            key: "freven.test:controls".to_string(),
        });
        assert_eq!(controls.output.input, vec![1, 2, 3]);
        assert_eq!(controls.output.view_yaw_deg_mdeg, 12_000);
        assert_eq!(controls.output.view_pitch_deg_mdeg, -3_000);
    }

    #[test]
    fn stateful_wasm_guest_macro_supports_provider_family_handlers() {
        std::thread_local! {
            static STORE: RefCell<StatefulGuestSessionStore<u32>> =
                const { RefCell::new(StatefulGuestSessionStore::new()) };
        }

        let module = stateful_wasm_guest!(
            @module
            guest_id: "freven.test.stateful.providers",
            session_factory: |_| 0_u32,
            session_store: &STORE,
            registration: {
                worldgen: "freven.test:flat" => provider_worldgen,
                character_controller: "freven.test:humanoid" => {
                    init: provider_character_init,
                    step: provider_character_step,
                },
                client_control_provider: "freven.test:controls" => provider_client_control
            }
        );

        assert_eq!(
            module.description().callbacks.providers,
            ProviderHooks {
                world: WorldProviderHooks { worldgen: true },
                avatar: AvatarProviderHooks {
                    character_controller: true,
                    client_control_provider: true,
                },
            }
        );
    }

    #[test]
    fn native_alloc_and_dealloc_helpers_round_trip() {
        let ptr = __private::native_guest_alloc(4);
        assert!(!ptr.is_null());

        unsafe {
            core::ptr::copy_nonoverlapping(b"rust".as_ptr(), ptr, 4);
        }

        __private::native_guest_dealloc(NativeGuestBuffer { ptr, len: 4 });
        __private::native_guest_dealloc(NativeGuestBuffer::empty());
    }

    #[test]
    fn authoritative_block_preserves_unsupported() {
        let _guard = install_test_runtime_service_hook(|request| {
            assert_eq!(
                request,
                WorldServiceRequest::Block(BlockServiceRequest::Query(
                    BlockQueryRequest::AuthoritativeBlock { pos: (1, 2, 3) }
                ))
            );
            WorldServiceResponse::Unsupported
        });

        assert_eq!(
            RuntimeServices.authoritative_block((1, 2, 3)),
            RuntimeQuerySupport::Unsupported
        );
    }

    #[test]
    fn authoritative_block_returns_supported_some() {
        let _guard = install_test_runtime_service_hook(|request| {
            assert_eq!(
                request,
                WorldServiceRequest::Block(BlockServiceRequest::Query(
                    BlockQueryRequest::AuthoritativeBlock { pos: (4, 5, 6) }
                ))
            );
            WorldServiceResponse::Block(BlockServiceResponse::Query(
                BlockQueryResponse::AuthoritativeBlock(Some(BlockRuntimeId(7))),
            ))
        });

        assert_eq!(
            RuntimeServices.authoritative_block((4, 5, 6)),
            RuntimeQuerySupport::Supported(Some(BlockRuntimeId(7)))
        );
    }

    #[test]
    fn authoritative_block_returns_supported_none() {
        let _guard = install_test_runtime_service_hook(|request| {
            assert_eq!(
                request,
                WorldServiceRequest::Block(BlockServiceRequest::Query(
                    BlockQueryRequest::AuthoritativeBlock { pos: (8, 9, 10) }
                ))
            );
            WorldServiceResponse::Block(BlockServiceResponse::Query(
                BlockQueryResponse::AuthoritativeBlock(None),
            ))
        });

        assert_eq!(
            RuntimeServices.authoritative_block((8, 9, 10)),
            RuntimeQuerySupport::Supported(None)
        );
    }
}
