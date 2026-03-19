use crate::ClientPlayerView;
use crate::ComponentId;
/// Mod message scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MessageScope {
    Global,
    Level { level_id: u32, stream_epoch: u32 },
}

/// Client outbound scope selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ClientOutboundMessageScope {
    Global,
    ActiveLevel,
}

/// Outbound client mod message payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientOutboundMessage {
    pub scope: ClientOutboundMessageScope,
    pub channel_id: u32,
    pub message_id: u32,
    pub seq: Option<u32>,
    pub payload: Vec<u8>,
}

/// Inbound client mod message payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientInboundMessage {
    pub scope: MessageScope,
    pub channel_id: u32,
    pub message_id: u32,
    pub seq: Option<u32>,
    pub payload: Vec<u8>,
}

/// Outbound server mod message payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerOutboundMessage {
    pub scope: MessageScope,
    pub channel_id: u32,
    pub message_id: u32,
    pub seq: Option<u32>,
    pub payload: Vec<u8>,
}

/// Inbound server mod message payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerInboundMessage {
    pub player_id: u64,
    pub scope: MessageScope,
    pub channel_id: u32,
    pub message_id: u32,
    pub seq: Option<u32>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("client message send failed: {message}")]
pub struct ClientMessageSendError {
    pub message: String,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("server message send failed: {message}")]
pub struct ServerMessageSendError {
    pub message: String,
}

/// Engine-provided client mod message surface.
pub trait ClientMessageProvider {
    fn send_msg(&mut self, msg: ClientOutboundMessage) -> Result<(), ClientMessageSendError>;
    fn poll_msg(&mut self) -> Option<ClientInboundMessage>;
}

/// Engine-provided server mod message surface.
pub trait ServerMessageProvider {
    fn send_to(
        &mut self,
        player_id: u64,
        msg: ServerOutboundMessage,
    ) -> Result<(), ServerMessageSendError>;
    fn poll_msg(&mut self) -> Option<ServerInboundMessage>;
}

/// Engine-provided client message send surface.
pub trait ClientMessageSender {
    fn send_msg(&mut self, msg: ClientOutboundMessage) -> Result<(), ClientMessageSendError>;
}

impl<T> ClientMessageSender for T
where
    T: ClientMessageProvider + ?Sized,
{
    fn send_msg(&mut self, msg: ClientOutboundMessage) -> Result<(), ClientMessageSendError> {
        ClientMessageProvider::send_msg(self, msg)
    }
}

/// Engine-provided server message send surface.
pub trait ServerMessageSender {
    fn send_to(
        &mut self,
        player_id: u64,
        msg: ServerOutboundMessage,
    ) -> Result<(), ServerMessageSendError>;
}

impl<T> ServerMessageSender for T
where
    T: ServerMessageProvider + ?Sized,
{
    fn send_to(
        &mut self,
        player_id: u64,
        msg: ServerOutboundMessage,
    ) -> Result<(), ServerMessageSendError> {
        ServerMessageProvider::send_to(self, player_id, msg)
    }
}

/// Engine-provided player presentation query surface.
pub trait ClientPlayerProvider {
    fn list_players(&self, out: &mut Vec<ClientPlayerView>);
    fn display_name_for(&self, player_id: u64) -> Option<String>;
    fn component_bytes_for(&self, player_id: u64, component_id: ComponentId) -> Option<&[u8]>;
    fn world_to_screen(&self, world_pos_m: (f32, f32, f32)) -> Option<(f32, f32)>;
}
