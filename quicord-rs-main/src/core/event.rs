/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use crate::core::client::Client;
use crate::core::storage::Storage;
use futures_util::FutureExt;
use rustc_hash::FxHashMap;
use std::future::Future;
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, Ordering},
};
use twilight_model::gateway::event::Event;

/// Future returned by event handlers.
pub type EventFuture = futures_util::future::BoxFuture<'static, anyhow::Result<()>>;

/// Function signature used by generated event handlers.
pub type EventHandler = fn(EventContext) -> EventFuture;

/// Context passed to event handlers.
#[derive(Clone)]
pub struct EventContext {
    /// The bot client.
    pub client: Client,
    bot_storage: Storage,
    /// The raw gateway event.
    pub event: Event,
    event_registry: EventRegistry,
}

impl EventContext {
    /// Creates a new event context.
    pub fn new(client: Client, storage: Storage, event: Event) -> Self {
        Self::with_registry(client, storage, event, EventRegistry::new())
    }

    /// Creates an event context associated with a bot's event registry.
    pub(crate) fn with_registry(
        client: Client,
        storage: Storage,
        event: Event,
        event_registry: EventRegistry,
    ) -> Self {
        Self {
            client,
            bot_storage: storage,
            event,
            event_registry,
        }
    }

    /// Returns a shared reference to a value from bot storage.
    pub fn storage<T: Send + Sync + 'static>(&self) -> anyhow::Result<&T> {
        self.bot_storage.require::<T>()
    }

    /// Registers an internal event hook for this bot.
    #[allow(dead_code)]
    pub(crate) fn register_event<F, Fut>(
        &self,
        event_type: impl AsRef<str>,
        handler: F,
    ) -> EventHookId
    where
        F: Fn(EventContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        self.event_registry.register(event_type, handler, false)
    }

    /// Registers an internal event hook that runs at most once for this bot.
    #[allow(dead_code)]
    pub(crate) fn register_event_once<F, Fut>(
        &self,
        event_type: impl AsRef<str>,
        handler: F,
    ) -> EventHookId
    where
        F: Fn(EventContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        self.event_registry.register_once(event_type, handler)
    }

    /// Removes an internal event hook from future dispatches.
    #[allow(dead_code)]
    pub(crate) fn unregister_event(&self, id: EventHookId) -> bool {
        self.event_registry.unregister(id)
    }
}

/// Metadata describing a gateway event handler.
pub struct EventHandlerMetadata {
    /// The event type name reported by Discord.
    pub event_type: &'static str,
    /// The handler invoked for the event.
    pub handler: EventHandler,
    /// Whether the handler should only be executed once.
    pub once: bool,
}

/// Distributed slice of all registered gateway event handlers.
#[linkme::distributed_slice]
pub static EVENT_HANDLERS: [EventHandlerMetadata];

/// Distributed slice of event handlers owned by the runtime.
///
/// Internal modules append entries with
/// `#[linkme::distributed_slice(crate::core::event::INTERNAL_EVENT_HANDLERS)]`.
#[linkme::distributed_slice]
pub(crate) static INTERNAL_EVENT_HANDLERS: [EventHandlerMetadata];

/// Identifier assigned to an event hook within one bot.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct EventHookId(u64);

/// A dynamically callable event handler.
type DynamicEventHandler = Arc<dyn Fn(EventContext) -> EventFuture + Send + Sync + 'static>;

/// One registered event hook.
pub(crate) struct RegisteredEventHook {
    id: EventHookId,
    once: bool,
    executed: AtomicBool,
    handler: DynamicEventHandler,
}

impl RegisteredEventHook {
    /// Claims a once hook for execution, or accepts every invocation for a normal hook.
    pub(crate) fn should_execute(&self) -> bool {
        !self.once || !self.executed.swap(true, Ordering::AcqRel)
    }

    /// Invokes the underlying handler.
    pub(crate) fn invoke(&self, context: EventContext) -> EventFuture {
        (self.handler)(context)
    }
}

/// A routed snapshot of all hooks for one gateway event.
pub(crate) struct EventRoute {
    pub(crate) event_type: String,
    pub(crate) hooks: Vec<Arc<RegisteredEventHook>>,
}

#[derive(Default)]
struct EventRegistryState {
    next_id: u64,
    hooks: FxHashMap<String, Vec<Arc<RegisteredEventHook>>>,
}

/// Registry shared by a bot and its internal event handlers.
#[derive(Clone, Default)]
pub(crate) struct EventRegistry {
    state: Arc<RwLock<EventRegistryState>>,
}

impl EventRegistry {
    /// Creates an empty registry.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Builds a registry from static event metadata.
    pub(crate) fn from_static<'a, I>(handlers: I) -> Self
    where
        I: IntoIterator<Item = &'a EventHandlerMetadata>,
    {
        let registry = Self::new();

        for metadata in handlers {
            registry.register_fn(metadata.event_type, metadata.handler, metadata.once);
        }

        registry
    }

    /// Registers an async closure for an event.
    pub(crate) fn register<F, Fut>(
        &self,
        event_type: impl AsRef<str>,
        handler: F,
        once: bool,
    ) -> EventHookId
    where
        F: Fn(EventContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let handler = Arc::new(move |context: EventContext| handler(context).boxed());
        self.register_handler(event_type.as_ref(), handler, once)
    }

    /// Registers an existing function-pointer handler.
    pub(crate) fn register_fn(
        &self,
        event_type: impl AsRef<str>,
        handler: EventHandler,
        once: bool,
    ) -> EventHookId {
        let handler = Arc::new(move |context: EventContext| handler(context));
        self.register_handler(event_type.as_ref(), handler, once)
    }

    /// Registers an async closure that runs at most once.
    pub(crate) fn register_once<F, Fut>(
        &self,
        event_type: impl AsRef<str>,
        handler: F,
    ) -> EventHookId
    where
        F: Fn(EventContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        self.register(event_type, handler, true)
    }

    /// Removes a hook from future event snapshots.
    pub(crate) fn unregister(&self, id: EventHookId) -> bool {
        let mut state = self.state.write().expect("Failed to access event registry");
        let mut empty_event_type = None;
        let mut removed = false;

        for (event_type, hooks) in &mut state.hooks {
            let original_len = hooks.len();
            hooks.retain(|hook| hook.id != id);
            if hooks.len() != original_len {
                removed = true;
                if hooks.is_empty() {
                    empty_event_type = Some(event_type.clone());
                }
                break;
            }
        }

        if let Some(event_type) = empty_event_type {
            state.hooks.remove(&event_type);
        }

        removed
    }

    /// Takes a stable snapshot of all hooks matching an event type.
    pub(crate) fn route(&self, event_type: &str) -> Option<EventRoute> {
        let event_type = normalize_event_type(event_type);
        let state = self.state.read().expect("Failed to access event registry");
        let hooks = state.hooks.get(&event_type)?.clone();

        Some(EventRoute { event_type, hooks })
    }

    fn register_handler(
        &self,
        event_type: &str,
        handler: DynamicEventHandler,
        once: bool,
    ) -> EventHookId {
        let event_type = normalize_event_type(event_type);
        let mut state = self.state.write().expect("Failed to access event registry");
        let id = EventHookId(state.next_id);
        state.next_id = state.next_id.wrapping_add(1);

        state
            .hooks
            .entry(event_type)
            .or_default()
            .push(Arc::new(RegisteredEventHook {
                id,
                once,
                executed: AtomicBool::new(false),
                handler,
            }));

        id
    }
}

fn normalize_event_type(event_type: &str) -> String {
    event_type.to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handler(_: EventContext) -> EventFuture {
        Box::pin(async { Ok(()) })
    }

    #[test]
    fn routes_all_handlers_for_one_event() {
        let registry = EventRegistry::new();
        registry.register_fn("ready", handler, false);
        registry.register_fn("READY", handler, false);

        let route = registry.route("Ready").expect("route should exist");
        assert_eq!(route.hooks.len(), 2);
    }

    #[test]
    fn unregister_removes_only_the_selected_handler() {
        let registry = EventRegistry::new();
        let first = registry.register_fn("ready", handler, false);
        registry.register_fn("ready", handler, false);

        assert!(registry.unregister(first));
        assert_eq!(registry.route("READY").unwrap().hooks.len(), 1);
        assert!(!registry.unregister(first));
    }

    #[test]
    fn once_handlers_are_claimed_once() {
        let registry = EventRegistry::new();
        registry.register_fn("ready", handler, true);

        let route = registry.route("READY").unwrap();
        assert!(route.hooks[0].should_execute());
        assert!(!route.hooks[0].should_execute());
    }
}
