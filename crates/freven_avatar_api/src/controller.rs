pub use freven_avatar_sdk_types::controller::*;

use freven_avatar_sdk_types::controller::{
    CharacterController as AvatarCharacterController,
    CharacterControllerFactory as AvatarCharacterControllerFactory,
    CharacterControllerInit as AvatarCharacterControllerInit,
};
use freven_world_api::{ModContext, ModRegistrationError};
use std::sync::Arc;

/// Numeric id for registered avatar controllers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CharacterControllerId(pub u32);

pub trait AvatarControllerRegistrationExt {
    fn register_character_controller(
        &mut self,
        key: &str,
        factory: impl Fn(AvatarCharacterControllerInit) -> Box<dyn AvatarCharacterController>
        + Send
        + Sync
        + 'static,
    ) -> Result<CharacterControllerId, ModRegistrationError>;

    #[doc(hidden)]
    fn __register_character_controller_factory(
        &mut self,
        key: &str,
        factory: AvatarCharacterControllerFactory,
    ) -> Result<CharacterControllerId, ModRegistrationError>;
}

impl AvatarControllerRegistrationExt for ModContext<'_> {
    fn register_character_controller(
        &mut self,
        key: &str,
        factory: impl Fn(AvatarCharacterControllerInit) -> Box<dyn AvatarCharacterController>
        + Send
        + Sync
        + 'static,
    ) -> Result<CharacterControllerId, ModRegistrationError> {
        self.__register_character_controller_factory(key, Arc::new(factory))
    }

    fn __register_character_controller_factory(
        &mut self,
        key: &str,
        factory: AvatarCharacterControllerFactory,
    ) -> Result<CharacterControllerId, ModRegistrationError> {
        self.__register_avatar_character_controller(key, Box::new(factory))
            .map(CharacterControllerId)
    }
}
