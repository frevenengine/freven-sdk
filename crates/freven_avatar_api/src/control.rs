pub use freven_avatar_sdk_types::control::*;

use freven_avatar_sdk_types::control::{
    ClientControlProvider as AvatarClientControlProvider,
    ClientControlProviderFactory as AvatarClientControlProviderFactory,
    ClientControlProviderInit as AvatarClientControlProviderInit,
};
use freven_world_api::{ModContext, ModRegistrationError};
use std::sync::Arc;

/// Numeric id for registered client control providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientControlProviderId(pub u32);

pub trait AvatarControlRegistrationExt {
    fn register_client_control_provider(
        &mut self,
        key: &str,
        factory: impl Fn(AvatarClientControlProviderInit) -> Box<dyn AvatarClientControlProvider>
        + Send
        + Sync
        + 'static,
    ) -> Result<ClientControlProviderId, ModRegistrationError>;

    #[doc(hidden)]
    fn __register_client_control_provider_factory(
        &mut self,
        key: &str,
        factory: AvatarClientControlProviderFactory,
    ) -> Result<ClientControlProviderId, ModRegistrationError>;
}

impl AvatarControlRegistrationExt for ModContext<'_> {
    fn register_client_control_provider(
        &mut self,
        key: &str,
        factory: impl Fn(AvatarClientControlProviderInit) -> Box<dyn AvatarClientControlProvider>
        + Send
        + Sync
        + 'static,
    ) -> Result<ClientControlProviderId, ModRegistrationError> {
        self.__register_client_control_provider_factory(key, Arc::new(factory))
    }

    fn __register_client_control_provider_factory(
        &mut self,
        key: &str,
        factory: AvatarClientControlProviderFactory,
    ) -> Result<ClientControlProviderId, ModRegistrationError> {
        self.__register_avatar_client_control_provider(key, Box::new(factory))
            .map(ClientControlProviderId)
    }
}
