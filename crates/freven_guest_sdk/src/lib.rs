//! High-level guest authoring helpers built on top of `freven_guest`.

extern crate alloc;

use alloc::vec::Vec;

pub use freven_guest::{
    ActionDeclaration, ActionInput, ActionOutcome, ActionResult, BlockDeclaration,
    CapabilityDeclaration, ChannelBudget, ChannelConfig, ChannelDeclaration, ChannelDirection,
    ChannelOrdering, ChannelReliability, ComponentCodec, ComponentDeclaration, EffectBatch,
    GUEST_CONTRACT_VERSION_1, GuestCallbacks, GuestDescription, GuestRegistration, GuestTransport,
    LifecycleAck, LifecycleHooks, MessageCodec, MessageDeclaration, MessageScope,
    NativeGuestBuffer, NativeGuestInput, NegotiationRequest, NegotiationResponse,
    ServerInboundMessage, ServerMessageInput, ServerMessageResult, ServerOutboundMessage,
    StartInput, TickInput, WorldEffect,
};
pub use freven_sdk_types::blocks::{BlockDef, RenderLayer};
use serde::de::DeserializeOwned;

type StartHandler = fn(&StartInput);
type TickHandler = fn(&TickInput);
type ActionHandler = fn(ActionContext<'_>) -> ActionResult;
type ServerMessageHandler = fn(ServerMessageContext<'_>) -> ServerMessageResponse;

pub struct GuestModule {
    guest_id: &'static str,
    blocks: Vec<BlockDeclaration>,
    components: Vec<ComponentDeclaration>,
    messages: Vec<MessageDeclaration>,
    channels: Vec<ChannelDeclaration>,
    actions: Vec<GuestAction>,
    capabilities: Vec<CapabilityDeclaration>,
    on_start_client: Option<StartHandler>,
    on_start_server: Option<StartHandler>,
    on_tick_client: Option<TickHandler>,
    on_tick_server: Option<TickHandler>,
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
            channels: Vec::new(),
            actions: Vec::new(),
            capabilities: Vec::new(),
            on_start_client: None,
            on_start_server: None,
            on_tick_client: None,
            on_tick_server: None,
            on_server_messages: None,
        }
    }

    #[must_use]
    pub fn register_block(mut self, key: &'static str, def: BlockDef) -> Self {
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
            server_messages: self.on_server_messages.is_some(),
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

    pub fn handle_start_client(&self, input: &StartInput) {
        if let Some(handler) = self.on_start_client {
            handler(input);
        }
    }

    pub fn handle_start_server(&self, input: &StartInput) {
        if let Some(handler) = self.on_start_server {
            handler(input);
        }
    }

    pub fn handle_tick_client(&self, input: &TickInput) {
        if let Some(handler) = self.on_tick_client {
            handler(input);
        }
    }

    pub fn handle_tick_server(&self, input: &TickInput) {
        if let Some(handler) = self.on_tick_server {
            handler(input);
        }
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
    pub fn player_position_m(&self) -> Option<[f32; 3]> {
        self.input.player_position_m
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
}

pub struct ActionResponse;

pub struct AppliedActionResponse {
    effects: EffectBatch,
}

pub struct RejectedActionResponse;

impl ActionResponse {
    #[must_use]
    pub fn applied() -> AppliedActionResponse {
        AppliedActionResponse {
            effects: EffectBatch::default(),
        }
    }

    #[must_use]
    pub fn rejected() -> RejectedActionResponse {
        RejectedActionResponse
    }
}

impl AppliedActionResponse {
    #[must_use]
    pub fn push_world_effect(mut self, effect: WorldEffect) -> Self {
        self.effects.world.push(effect);
        self
    }

    #[must_use]
    pub fn set_block(self, pos: (i32, i32, i32), block_id: u8) -> Self {
        self.push_world_effect(WorldEffect::SetBlock { pos, block_id })
    }

    #[must_use]
    pub fn finish(self) -> ActionResult {
        ActionResult {
            outcome: ActionOutcome::Applied,
            effects: self.effects,
        }
    }
}

impl RejectedActionResponse {
    #[must_use]
    pub fn finish(self) -> ActionResult {
        ActionResult {
            outcome: ActionOutcome::Rejected,
            effects: EffectBatch::default(),
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
}

#[derive(Default)]
pub struct ServerMessageResponse {
    outbound: Vec<ServerOutboundMessage>,
}

impl ServerMessageResponse {
    #[must_use]
    pub fn send_to(mut self, message: ServerOutboundMessage) -> Self {
        self.outbound.push(message);
        self
    }

    #[must_use]
    pub fn finish(self) -> ServerMessageResult {
        ServerMessageResult {
            outbound: self.outbound,
        }
    }
}

#[doc(hidden)]
pub mod __private {
    use super::*;

    fn module_negotiate_bytes(
        module: &GuestModule,
        input: &[u8],
        transport: GuestTransport,
    ) -> Vec<u8> {
        if !input.is_empty() {
            let request: NegotiationRequest =
                postcard::from_bytes(input).expect("valid negotiation request");
            assert_eq!(request.transport, transport);
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

    fn module_start_client_bytes(module: &GuestModule, input: &[u8]) -> Vec<u8> {
        let input = decode_default_input::<StartInput>(input);
        module.handle_start_client(&input);
        encode_lifecycle_ack_bytes()
    }

    fn module_start_server_bytes(module: &GuestModule, input: &[u8]) -> Vec<u8> {
        let input = decode_default_input::<StartInput>(input);
        module.handle_start_server(&input);
        encode_lifecycle_ack_bytes()
    }

    fn module_tick_client_bytes(module: &GuestModule, input: &[u8]) -> Vec<u8> {
        let input = decode_required_input::<TickInput>(input);
        module.handle_tick_client(&input);
        encode_lifecycle_ack_bytes()
    }

    fn module_tick_server_bytes(module: &GuestModule, input: &[u8]) -> Vec<u8> {
        let input = decode_required_input::<TickInput>(input);
        module.handle_tick_server(&input);
        encode_lifecycle_ack_bytes()
    }

    fn module_handle_action_bytes(module: &GuestModule, input: &[u8]) -> Vec<u8> {
        assert!(!input.is_empty(), "guest input must not be empty");
        let input: ActionInput<'_> = postcard::from_bytes(input).expect("valid action input");

        let result = module.handle_action(input);
        postcard::to_allocvec(&result).expect("guest encoding must succeed")
    }

    fn module_server_messages_bytes(module: &GuestModule, input: &[u8]) -> Vec<u8> {
        let input = decode_required_input::<ServerMessageInput>(input);
        postcard::to_allocvec(&module.handle_server_messages(input))
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

    pub fn wasm_guest_negotiate(module: &GuestModule, ptr: u32, len: u32) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_negotiate_bytes(
                module,
                input,
                GuestTransport::WasmPtrLenV1,
            ))
        })
    }

    pub fn wasm_guest_start_client(module: &GuestModule, ptr: u32, len: u32) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_start_client_bytes(module, input))
        })
    }

    pub fn wasm_guest_start_server(module: &GuestModule, ptr: u32, len: u32) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_start_server_bytes(module, input))
        })
    }

    pub fn wasm_guest_tick_client(module: &GuestModule, ptr: u32, len: u32) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_tick_client_bytes(module, input))
        })
    }

    pub fn wasm_guest_tick_server(module: &GuestModule, ptr: u32, len: u32) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_tick_server_bytes(module, input))
        })
    }

    pub fn wasm_guest_handle_action(module: &GuestModule, ptr: u32, len: u32) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_handle_action_bytes(module, input))
        })
    }

    pub fn wasm_guest_server_messages(module: &GuestModule, ptr: u32, len: u32) -> u64 {
        with_wasm_input_bytes(ptr, len, |input| {
            encode_to_wasm_guest(&module_server_messages_bytes(module, input))
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

    pub fn native_guest_negotiate(
        module: &GuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_negotiate_bytes(
                module,
                input,
                GuestTransport::NativeInProcessV1,
            ))
        })
    }

    pub fn native_guest_start_client(
        module: &GuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_start_client_bytes(module, input))
        })
    }

    pub fn native_guest_start_server(
        module: &GuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_start_server_bytes(module, input))
        })
    }

    pub fn native_guest_tick_client(
        module: &GuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_tick_client_bytes(module, input))
        })
    }

    pub fn native_guest_tick_server(
        module: &GuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_tick_server_bytes(module, input))
        })
    }

    pub fn native_guest_handle_action(
        module: &GuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_handle_action_bytes(module, input))
        })
    }

    pub fn native_guest_server_messages(
        module: &GuestModule,
        input: NativeGuestInput,
    ) -> NativeGuestBuffer {
        with_native_input_bytes(input, |input| {
            encode_to_native_guest(module_server_messages_bytes(module, input))
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

    fn encode_lifecycle_ack_bytes() -> Vec<u8> {
        postcard::to_allocvec(&LifecycleAck::default()).expect("guest encoding must succeed")
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
        module: &GuestModule,
        lifecycle: LifecycleHooks,
        action: bool,
        server_messages: bool,
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
            callbacks.server_messages, server_messages,
            "freven_guest_sdk server message export does not match GuestModule::description()",
        );
    }
}

#[macro_export]
macro_rules! export_wasm_guest {
    (
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),* $(,)?])?
        $(, actions: $actions:tt)?
        $(, server_messages: $server_messages:tt)?
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
                $crate::export_wasm_guest!(@bool $($server_messages)?),
            );
            $crate::__private::wasm_guest_negotiate(&module, ptr, len)
        }

        $crate::export_wasm_guest!(@maybe_export $factory, start_client, $($($lifecycle),*)?);
        $crate::export_wasm_guest!(@maybe_export $factory, start_server, $($($lifecycle),*)?);
        $crate::export_wasm_guest!(@maybe_export $factory, tick_client, $($($lifecycle),*)?);
        $crate::export_wasm_guest!(@maybe_export $factory, tick_server, $($($lifecycle),*)?);
        $crate::export_wasm_guest!(@maybe_export_action $factory, $($actions)?);
        $crate::export_wasm_guest!(@maybe_export_server_messages $factory, $($server_messages)?);
    };

    (@lifecycle_struct $($hook:ident),*) => {{
        let mut hooks = $crate::LifecycleHooks::default();
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
}

#[macro_export]
macro_rules! export_native_guest {
    (
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),* $(,)?])?
        $(, actions: $actions:tt)?
        $(, server_messages: $server_messages:tt)?
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
        pub extern "C" fn freven_guest_negotiate(
            input: $crate::NativeGuestInput,
        ) -> $crate::NativeGuestBuffer {
            let module = $factory();
            $crate::__private::assert_export_surface(
                &module,
                $crate::export_native_guest!(@lifecycle_struct $($($lifecycle),*)?),
                $crate::export_native_guest!(@bool $($actions)?),
                $crate::export_native_guest!(@bool $($server_messages)?),
            );
            $crate::__private::native_guest_negotiate(&module, input)
        }

        $crate::export_native_guest!(@maybe_export $factory, start_client, $($($lifecycle),*)?);
        $crate::export_native_guest!(@maybe_export $factory, start_server, $($($lifecycle),*)?);
        $crate::export_native_guest!(@maybe_export $factory, tick_client, $($($lifecycle),*)?);
        $crate::export_native_guest!(@maybe_export $factory, tick_server, $($($lifecycle),*)?);
        $crate::export_native_guest!(@maybe_export_action $factory, $($actions)?);
        $crate::export_native_guest!(@maybe_export_server_messages $factory, $($server_messages)?);
    };

    (@lifecycle_struct $($hook:ident),*) => {{
        let mut hooks = $crate::LifecycleHooks::default();
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
}

#[macro_export]
macro_rules! wasm_guest {
    (
        guest_id: $guest_id:expr
        $(, registration: { $($registration:tt)* })?
        $(, lifecycle: { $($lifecycle:ident : $lifecycle_handler:expr),* $(,)? })?
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
            $(, lifecycle: [$($lifecycle),*])?
            $(, server_messages: [$server_messages_handler])?
            $(, actions: [$($action_key),*])?
        );
    };

    (
        @module
        guest_id: $guest_id:expr
        $(, registration: { $($registration:tt)* })?
        $(, lifecycle: { $($lifecycle:ident : $lifecycle_handler:expr),* $(,)? })?
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

    (@export factory: $factory:path $(, lifecycle: [$($lifecycle:ident),*])?) => {
        $crate::export_wasm_guest!(
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
        );
    };
    (@export factory: $factory:path $(, lifecycle: [$($lifecycle:ident),*])?, server_messages: [$handler:expr]) => {
        $crate::export_wasm_guest!(
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
            , server_messages: true
        );
    };
    (@export factory: $factory:path $(, lifecycle: [$($lifecycle:ident),*])?, actions: []) => {
        $crate::export_wasm_guest!(
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
        );
    };
    (@export factory: $factory:path $(, lifecycle: [$($lifecycle:ident),*])?, actions: [$first:expr $(, $rest:expr)*]) => {
        $crate::export_wasm_guest!(
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
            , actions: true
        );
    };
    (@export factory: $factory:path $(, lifecycle: [$($lifecycle:ident),*])?, server_messages: [$handler:expr], actions: []) => {
        $crate::export_wasm_guest!(
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
            , server_messages: true
        );
    };
    (@export factory: $factory:path $(, lifecycle: [$($lifecycle:ident),*])?, server_messages: [$handler:expr], actions: [$first:expr $(, $rest:expr)*]) => {
        $crate::export_wasm_guest!(
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
            , actions: true
            , server_messages: true
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
    (@registration $module:expr, channel: $key:expr => $config:expr $(, $($rest:tt)*)?) => {
        $crate::wasm_guest!(@registration $module.register_channel($key, $config) $(, $($rest)*)?)
    };
    (@registration $module:expr, capability: $key:expr $(, $($rest:tt)*)?) => {
        $crate::wasm_guest!(@registration $module.declare_capability($key) $(, $($rest)*)?)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

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
                BlockDef {
                    is_solid: true,
                    is_opaque: true,
                    render_layer: RenderLayer::Opaque,
                    debug_tint_rgba: 0,
                    material_id: 7,
                },
            )
            .register_component("freven.test:name", ComponentCodec::RawBytes)
            .register_message("freven.test:echo", MessageCodec::RawBytes)
            .register_channel("freven.test:echo", message_channel())
            .declare_capability("max_call_millis")
            .on_start_server(|_| {})
            .on_tick_server(|_| {})
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
                ActionResponse::applied().set_block((1, 2, 3), 9).finish()
            })
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
        assert_eq!(description.registration.channels.len(), 1);
        assert_eq!(description.registration.actions.len(), 1);
        assert_eq!(description.registration.capabilities.len(), 1);
        assert!(description.callbacks.lifecycle.start_server);
        assert!(description.callbacks.lifecycle.tick_server);
        assert!(description.callbacks.action);
        assert!(description.callbacks.server_messages);
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
            player_position_m: Some([1.0, 2.0, 3.0]),
            payload: &[],
        });

        assert_eq!(result.outcome, ActionOutcome::Rejected);
        assert!(result.effects.is_empty());
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
        assert_eq!(result.outbound.len(), 1);
        assert_eq!(result.outbound[0].payload, b"hello");
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
        assert!(result.effects.is_empty());
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
            true,
        );
    }

    #[test]
    #[should_panic(expected = "freven_guest_sdk export lifecycle does not match")]
    fn export_surface_assertion_rejects_lifecycle_mismatch() {
        __private::assert_export_surface(&module(), LifecycleHooks::default(), true, true);
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
            true,
        );
    }

    #[test]
    #[should_panic(expected = "freven_guest_sdk server message export does not match")]
    fn export_surface_assertion_rejects_server_message_mismatch() {
        __private::assert_export_surface(
            &module(),
            LifecycleHooks {
                start_server: true,
                tick_server: true,
                ..Default::default()
            },
            true,
            false,
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
                start_server: |_| {},
                tick_server: |_| {}
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
        assert!(description.callbacks.server_messages);
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
}
