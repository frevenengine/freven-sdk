//! High-level guest authoring helpers built on top of `freven_guest`.

extern crate alloc;

use alloc::vec::Vec;

pub use freven_guest::{
    ActionBinding, ActionInput, ActionOutcome, ActionResult, EffectBatch, GUEST_CONTRACT_VERSION_1,
    GuestDescription, GuestTransport, LifecycleAck, LifecycleHooks, NativeGuestBuffer,
    NativeGuestInput, NegotiationRequest, NegotiationResponse, StartInput, TickInput, WorldEffect,
};
use serde::de::DeserializeOwned;

type StartHandler = fn(&StartInput);
type TickHandler = fn(&TickInput);
type ActionHandler = fn(ActionContext<'_>) -> ActionResponse;

pub struct GuestModule {
    guest_id: &'static str,
    actions: Vec<GuestAction>,
    on_start_client: Option<StartHandler>,
    on_start_server: Option<StartHandler>,
    on_tick_client: Option<TickHandler>,
    on_tick_server: Option<TickHandler>,
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
            actions: Vec::new(),
            on_start_client: None,
            on_start_server: None,
            on_tick_client: None,
            on_tick_server: None,
        }
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
    pub fn action(mut self, key: &'static str, binding_id: u32, handler: ActionHandler) -> Self {
        assert!(
            !key.trim().is_empty(),
            "freven_guest_sdk action key must not be empty"
        );
        assert!(
            self.actions.iter().all(|action| action.key != key),
            "freven_guest_sdk action key '{key}' was registered more than once"
        );
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
    pub fn description(&self) -> GuestDescription {
        GuestDescription {
            guest_id: self.guest_id.to_string(),
            lifecycle: self.lifecycle_hooks(),
            action_entrypoint: !self.actions.is_empty(),
            actions: self
                .actions
                .iter()
                .map(|action| ActionBinding {
                    key: action.key.to_string(),
                    binding_id: action.binding_id,
                })
                .collect(),
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

        (action.handler)(ActionContext { input }).finish()
    }
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

pub struct ActionResponse {
    outcome: ActionOutcome,
    effects: EffectBatch,
}

impl ActionResponse {
    #[must_use]
    pub fn applied() -> Self {
        Self {
            outcome: ActionOutcome::Applied,
            effects: EffectBatch::default(),
        }
    }

    #[must_use]
    pub fn rejected() -> Self {
        Self {
            outcome: ActionOutcome::Rejected,
            effects: EffectBatch::default(),
        }
    }

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
            outcome: self.outcome,
            effects: self.effects,
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
        let input = if input.is_empty() {
            ActionInput {
                binding_id: 0,
                player_id: 0,
                level_id: 0,
                stream_epoch: 0,
                action_seq: 0,
                at_input_seq: 0,
                payload: &[],
            }
        } else {
            postcard::from_bytes(input).expect("valid action input")
        };

        let result = module.handle_action(input);
        postcard::to_allocvec(&result).expect("guest encoding must succeed")
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

    // Native guest buffers in the SDK are owned as exact-sized boxed slices.
    // That keeps deallocation dependent only on (ptr, len) and avoids any Vec
    // capacity-coupling in the native ABI helper path.
    // The canonical empty native buffer is null + zero; these helpers never
    // intentionally produce non-null + zero.
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
        // Canonical empty native buffers are null + zero. Invalid non-null + zero
        // is treated as no-op here rather than reconstructing ownership from a
        // malformed buffer shape.
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
        action_entrypoint: bool,
    ) {
        let description = module.description();
        assert_eq!(
            description.lifecycle, lifecycle,
            "freven_guest_sdk export lifecycle does not match GuestModule::description()",
        );
        assert_eq!(
            description.action_entrypoint, action_entrypoint,
            "freven_guest_sdk action export does not match GuestModule::description()",
        );
    }
}

/// Lower-level Wasm export wiring for cases where you intentionally manage a
/// `GuestModule` factory and export surface separately.
#[macro_export]
macro_rules! export_wasm_guest {
    (
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),* $(,)?])?
        $(, actions: $actions:tt)?
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
                $crate::export_wasm_guest!(@actions_bool $($actions)?),
            );
            $crate::__private::wasm_guest_negotiate(&module, ptr, len)
        }

        $crate::export_wasm_guest!(@maybe_export $factory, start_client, $($($lifecycle),*)?);
        $crate::export_wasm_guest!(@maybe_export $factory, start_server, $($($lifecycle),*)?);
        $crate::export_wasm_guest!(@maybe_export $factory, tick_client, $($($lifecycle),*)?);
        $crate::export_wasm_guest!(@maybe_export $factory, tick_server, $($($lifecycle),*)?);
        $crate::export_wasm_guest!(@maybe_export_action $factory, $($actions)?);
    };

    (@lifecycle_struct $($hook:ident),*) => {{
        let mut hooks = $crate::LifecycleHooks::default();
        $(hooks.$hook = true;)*
        hooks
    }};

    (@actions_bool true) => {
        true
    };
    (@actions_bool false) => {
        false
    };
    (@actions_bool) => {
        false
    };

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
}

/// Lower-level native export wiring for cases where you intentionally manage a
/// `GuestModule` factory and export surface separately.
#[macro_export]
macro_rules! export_native_guest {
    (
        factory: $factory:path
        $(, lifecycle: [$($lifecycle:ident),* $(,)?])?
        $(, actions: $actions:tt)?
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
                $crate::export_native_guest!(@actions_bool $($actions)?),
            );
            $crate::__private::native_guest_negotiate(&module, input)
        }

        $crate::export_native_guest!(@maybe_export $factory, start_client, $($($lifecycle),*)?);
        $crate::export_native_guest!(@maybe_export $factory, start_server, $($($lifecycle),*)?);
        $crate::export_native_guest!(@maybe_export $factory, tick_client, $($($lifecycle),*)?);
        $crate::export_native_guest!(@maybe_export $factory, tick_server, $($($lifecycle),*)?);
        $crate::export_native_guest!(@maybe_export_action $factory, $($actions)?);
    };

    (@lifecycle_struct $($hook:ident),*) => {{
        let mut hooks = $crate::LifecycleHooks::default();
        $(hooks.$hook = true;)*
        hooks
    }};

    (@actions_bool true) => {
        true
    };
    (@actions_bool false) => {
        false
    };
    (@actions_bool) => {
        false
    };

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
}

/// Defines the canonical guest description and the Wasm export surface from one
/// declarative source of truth.
#[macro_export]
macro_rules! wasm_guest {
    (
        guest_id: $guest_id:expr
        $(, lifecycle: { $($lifecycle:ident : $lifecycle_handler:expr),* $(,)? })?
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
                $(, lifecycle: { $($lifecycle : $lifecycle_handler),* })?
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
            $(, actions: [$($action_key),*])?
        );
    };

    (
        @module
        guest_id: $guest_id:expr
        $(, lifecycle: { $($lifecycle:ident : $lifecycle_handler:expr),* $(,)? })?
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
            $(
                let module = module.action($action_key, $binding_id, $action_handler);
            )*
        )?
        module
    }};

    (@register_lifecycle $module:ident, start_client, $handler:expr) => {
        $module.on_start_client($handler)
    };
    (@register_lifecycle $module:ident, start_server, $handler:expr) => {
        $module.on_start_server($handler)
    };
    (@register_lifecycle $module:ident, tick_client, $handler:expr) => {
        $module.on_tick_client($handler)
    };
    (@register_lifecycle $module:ident, tick_server, $handler:expr) => {
        $module.on_tick_server($handler)
    };

    (@export factory: $factory:path $(, lifecycle: [$($lifecycle:ident),*])?) => {
        $crate::export_wasm_guest!(
            factory: $factory
            $(, lifecycle: [$($lifecycle),*])?
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn module() -> GuestModule {
        GuestModule::new("freven.test.guest")
            .on_start_server(|_| {})
            .on_tick_server(|_| {})
            .action("freven.test:place_block", 7, |_| {
                ActionResponse::applied().set_block((1, 2, 3), 9)
            })
    }

    #[test]
    fn description_reflects_registered_lifecycle_and_actions() {
        let description = module().description();
        assert_eq!(description.guest_id, "freven.test.guest");
        assert!(description.lifecycle.start_server);
        assert!(description.lifecycle.tick_server);
        assert!(!description.lifecycle.start_client);
        assert!(description.action_entrypoint);
        assert_eq!(description.actions.len(), 1);
        assert_eq!(description.actions[0].binding_id, 7);
        assert_eq!(description.actions[0].key, "freven.test:place_block");
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
        assert!(result.effects.is_empty());
    }

    #[test]
    #[should_panic(expected = "guest_id must not be empty")]
    fn empty_guest_id_is_rejected() {
        let _ = GuestModule::new("  ");
    }

    #[test]
    #[should_panic(expected = "action key must not be empty")]
    fn empty_action_key_is_rejected() {
        let _ =
            GuestModule::new("freven.test.guest").action(" ", 1, |_| ActionResponse::rejected());
    }

    #[test]
    #[should_panic(expected = "registered more than once")]
    fn duplicate_action_key_is_rejected() {
        let _ = GuestModule::new("freven.test.guest")
            .action("freven.test:dup", 1, |_| ActionResponse::rejected())
            .action("freven.test:dup", 2, |_| ActionResponse::rejected());
    }

    #[test]
    #[should_panic(expected = "binding id 1 was registered more than once")]
    fn duplicate_binding_id_is_rejected() {
        let _ = GuestModule::new("freven.test.guest")
            .action("freven.test:first", 1, |_| ActionResponse::rejected())
            .action("freven.test:second", 1, |_| ActionResponse::rejected());
    }

    #[test]
    fn export_surface_assertion_matches_description() {
        let module = module();
        __private::assert_export_surface(
            &module,
            LifecycleHooks {
                start_server: true,
                tick_server: true,
                ..Default::default()
            },
            true,
        );
    }

    #[test]
    #[should_panic(expected = "export lifecycle does not match")]
    fn export_surface_assertion_rejects_lifecycle_mismatch() {
        let module = module();
        __private::assert_export_surface(&module, LifecycleHooks::default(), true);
    }

    #[test]
    #[should_panic(expected = "action export does not match")]
    fn export_surface_assertion_rejects_action_mismatch() {
        let module = module();
        __private::assert_export_surface(
            &module,
            LifecycleHooks {
                start_server: true,
                tick_server: true,
                ..Default::default()
            },
            false,
        );
    }

    fn test_start_server(_: &StartInput) {}

    fn test_tick_server(_: &TickInput) {}

    fn test_handle_action(_: ActionContext<'_>) -> ActionResponse {
        ActionResponse::applied()
    }

    #[test]
    fn wasm_guest_macro_module_description_matches_declared_surface() {
        let module = crate::wasm_guest!(
            @module
            guest_id: "freven.test.single_source",
            lifecycle: {
                start_server: test_start_server,
                tick_server: test_tick_server,
            },
            actions: {
                "freven.test:macro_action" => {
                    binding_id: 11,
                    handler: test_handle_action,
                },
            },
        );

        assert_eq!(module.guest_id(), "freven.test.single_source");
        assert_eq!(
            module.lifecycle_hooks(),
            LifecycleHooks {
                start_server: true,
                tick_server: true,
                ..Default::default()
            }
        );

        let description = module.description();
        assert!(description.action_entrypoint);
        assert_eq!(description.actions.len(), 1);
        assert_eq!(description.actions[0].key, "freven.test:macro_action");
        assert_eq!(description.actions[0].binding_id, 11);
    }

    #[test]
    fn wasm_guest_macro_supports_guests_without_actions() {
        let module = crate::wasm_guest!(
            @module
            guest_id: "freven.test.lifecycle_only",
            lifecycle: {
                start_server: test_start_server,
            },
            actions: {},
        );

        let description = module.description();
        assert!(!description.action_entrypoint);
        assert!(description.actions.is_empty());
        assert_eq!(
            description.lifecycle,
            LifecycleHooks {
                start_server: true,
                ..Default::default()
            }
        );
    }

    #[test]
    fn native_guest_alloc_dealloc_round_trips_exact_sized_buffer() {
        let ptr = __private::native_guest_alloc(16);
        assert!(!ptr.is_null());

        unsafe {
            core::ptr::write_bytes(ptr, 0xAB, 16);
        }

        __private::native_guest_dealloc(NativeGuestBuffer { ptr, len: 16 });
    }

    #[test]
    fn native_guest_dealloc_accepts_empty_buffer() {
        __private::native_guest_dealloc(NativeGuestBuffer::empty());
    }
}
