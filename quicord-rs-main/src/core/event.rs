/*
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use crate::core::client::Client;
use twilight_model::gateway::event::Event;

pub type EventHandler =
    fn(EventContext) -> futures_util::future::BoxFuture<'static, anyhow::Result<()>>;

#[derive(Clone)]
pub struct EventContext {
    pub client: Client,
    pub event: Event,
}

impl EventContext {
    pub fn new(client: Client, event: Event) -> Self {
        Self { client, event }
    }
}

pub struct EventHandlerMetadata {
    pub event_type: &'static str,
    pub handler: EventHandler,
}

#[linkme::distributed_slice]
pub static EVENT_HANDLERS: [EventHandlerMetadata];
