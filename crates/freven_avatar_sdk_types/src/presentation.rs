use crate::identity::PlayerId;
use freven_world_api::ComponentId;

pub use freven_world_guest::ClientPlayerView as GuestClientPlayerView;

/// Lightweight avatar view for client-side presentation mods.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientPlayerView {
    pub player_id: PlayerId,
    pub world_pos_m: (f32, f32, f32),
    pub is_local: bool,
}

/// Engine-provided avatar presentation query surface.
pub trait ClientPlayerProvider {
    fn list_players(&self, out: &mut Vec<ClientPlayerView>);
    fn display_name_for(&self, player_id: PlayerId) -> Option<String>;
    fn component_bytes_for(&self, player_id: PlayerId, component_id: ComponentId) -> Option<&[u8]>;
    fn world_to_screen(&self, world_pos_m: (f32, f32, f32)) -> Option<(f32, f32)>;
}
