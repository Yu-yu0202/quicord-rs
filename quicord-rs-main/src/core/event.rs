/*
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use crate::core::client::Client;
use twilight_model::gateway::event::Event;

/// Asynchronous handler signature for gateway events.
pub type EventHandler =
    fn(EventContext) -> futures_util::future::BoxFuture<'static, anyhow::Result<()>>;

/// Context passed to event handlers.
#[derive(Clone)]
pub struct EventContext {
    /// The bot client.
    pub client: Client,
    /// The raw gateway event.
    pub event: Event,
}

impl EventContext {
    /// Creates a new event context.
    pub fn new(client: Client, event: Event) -> Self {
        Self { client, event }
    }
}

/// Metadata describing a gateway event handler.
pub struct EventHandlerMetadata {
    /// The event type name reported by Discord.
    pub event_type: &'static str,
    /// The handler invoked for the event.
    pub handler: EventHandler,
}

/// Distributed slice of all registered gateway event handlers.
#[linkme::distributed_slice]
pub static EVENT_HANDLERS: [EventHandlerMetadata];
