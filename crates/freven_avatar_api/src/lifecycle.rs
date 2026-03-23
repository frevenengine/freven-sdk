use std::time::Duration;

use freven_avatar_sdk_types::{control::ClientInputProvider, presentation::ClientPlayerProvider};
use freven_block_api::ClientCameraHitProvider;
use freven_mod_api::{LogLevel, emit_log};
use freven_world_api::{ClientInteractionProvider, ModContext, Services};

/// Lifecycle callback executed once when the client side starts.
pub type StartClientHook = for<'a> fn(&mut ClientApi<'a>);

/// Lifecycle callback executed on each client tick.
pub type TickClientHook = for<'a> fn(&mut ClientTickApi<'a>);

/// Avatar-facing client lifecycle API.
pub struct ClientApi<'a> {
    pub services: &'a mut dyn Services,
    pub input: &'a mut dyn ClientInputProvider,
    pub camera: &'a mut dyn ClientCameraHitProvider,
    pub interaction: &'a mut dyn ClientInteractionProvider,
    pub players: &'a mut dyn ClientPlayerProvider,
}

impl<'a> ClientApi<'a> {
    #[must_use]
    pub fn new(
        services: &'a mut dyn Services,
        input: &'a mut dyn ClientInputProvider,
        camera: &'a mut dyn ClientCameraHitProvider,
        interaction: &'a mut dyn ClientInteractionProvider,
        players: &'a mut dyn ClientPlayerProvider,
    ) -> Self {
        Self {
            services,
            input,
            camera,
            interaction,
            players,
        }
    }

    #[must_use]
    pub fn reborrow(&mut self) -> ClientApi<'_> {
        ClientApi {
            services: self.services,
            input: self.input,
            camera: self.camera,
            interaction: self.interaction,
            players: self.players,
        }
    }

    pub fn log(&mut self, level: LogLevel, message: impl AsRef<str>) {
        let _ = &self.services;
        emit_log(level, message);
    }
}

/// Avatar-facing client tick lifecycle API.
pub struct ClientTickApi<'a> {
    pub tick: u64,
    pub dt: Duration,
    pub client: ClientApi<'a>,
}

impl<'a> ClientTickApi<'a> {
    #[must_use]
    pub fn new(tick: u64, dt: Duration, client: ClientApi<'a>) -> Self {
        Self { tick, dt, client }
    }

    pub fn log(&mut self, level: LogLevel, message: impl AsRef<str>) {
        self.client.log(level, message);
    }
}

pub trait AvatarLifecycleRegistrationExt {
    fn on_start_client(&mut self, hook: StartClientHook);
    fn on_tick_client(&mut self, hook: TickClientHook);
}

impl AvatarLifecycleRegistrationExt for ModContext<'_> {
    fn on_start_client(&mut self, hook: StartClientHook) {
        self.__on_avatar_start_client(Box::new(hook));
    }

    fn on_tick_client(&mut self, hook: TickClientHook) {
        self.__on_avatar_tick_client(Box::new(hook));
    }
}
