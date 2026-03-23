use std::time::Duration;

use crate::{
    messages::{
        ClientInboundMessage, ClientMessageSender, ServerInboundMessage, ServerMessageSender,
    },
    services::Services,
};
use freven_mod_api::{LogLevel, emit_log};

/// Lifecycle callback executed once when the server side starts.
pub type StartServerHook = for<'a> fn(&mut ServerApi<'a>);

/// Lifecycle callback executed on each server tick.
pub type TickServerHook = for<'a> fn(&mut ServerTickApi<'a>);

/// Message callback executed on each client message dispatch phase.
pub type ClientMessagesHook = for<'a> fn(&mut ClientMessagesApi<'a>);

/// Message callback executed on each server message dispatch phase.
pub type ServerMessagesHook = for<'a> fn(&mut ServerMessagesApi<'a>);

/// Server-side lifecycle API.
pub struct ServerApi<'a> {
    pub services: &'a mut dyn Services,
}

impl<'a> ServerApi<'a> {
    #[must_use]
    pub fn new(services: &'a mut dyn Services) -> Self {
        Self { services }
    }

    pub fn log(&mut self, level: LogLevel, message: impl AsRef<str>) {
        let _ = &self.services;
        emit_log(level, message);
    }
}

/// Client-side message dispatch context.
pub struct ClientMessagesApi<'a> {
    pub tick: u64,
    pub dt: Duration,
    pub services: &'a mut dyn Services,
    pub inbound: &'a [ClientInboundMessage],
    pub sender: &'a mut dyn ClientMessageSender,
}

impl<'a> ClientMessagesApi<'a> {
    #[must_use]
    pub fn new(
        tick: u64,
        dt: Duration,
        services: &'a mut dyn Services,
        inbound: &'a [ClientInboundMessage],
        sender: &'a mut dyn ClientMessageSender,
    ) -> Self {
        Self {
            tick,
            dt,
            services,
            inbound,
            sender,
        }
    }

    pub fn log(&mut self, level: LogLevel, message: impl AsRef<str>) {
        let _ = &self.services;
        emit_log(level, message);
    }
}

/// Server-side message dispatch context.
pub struct ServerMessagesApi<'a> {
    pub tick: u64,
    pub dt: Duration,
    pub services: &'a mut dyn Services,
    pub inbound: &'a [ServerInboundMessage],
    pub sender: &'a mut dyn ServerMessageSender,
}

impl<'a> ServerMessagesApi<'a> {
    #[must_use]
    pub fn new(
        tick: u64,
        dt: Duration,
        services: &'a mut dyn Services,
        inbound: &'a [ServerInboundMessage],
        sender: &'a mut dyn ServerMessageSender,
    ) -> Self {
        Self {
            tick,
            dt,
            services,
            inbound,
            sender,
        }
    }

    pub fn log(&mut self, level: LogLevel, message: impl AsRef<str>) {
        let _ = &self.services;
        emit_log(level, message);
    }
}

/// Server-side lifecycle tick context.
pub struct ServerTickApi<'a> {
    pub tick: u64,
    pub dt: Duration,
    pub server: ServerApi<'a>,
}

impl<'a> ServerTickApi<'a> {
    #[must_use]
    pub fn new(tick: u64, dt: Duration, server: ServerApi<'a>) -> Self {
        Self { tick, dt, server }
    }

    pub fn log(&mut self, level: LogLevel, message: impl AsRef<str>) {
        self.server.log(level, message);
    }
}
