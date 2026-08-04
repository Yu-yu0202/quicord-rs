/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0.
 */

use crate::core::event::{EventContext, EventHookId, EventRegistry};
use crate::core::storage::Storage;
use std::future::Future;

/// State shared by event and interaction handler contexts.
#[derive(Clone)]
pub(crate) struct HandlerContext {
    storage: Storage,
    event_registry: EventRegistry,
}

impl HandlerContext {
    pub(crate) fn new(storage: Storage, event_registry: EventRegistry) -> Self {
        Self {
            storage,
            event_registry,
        }
    }

    pub(crate) fn storage<T: Send + Sync + 'static>(&self) -> anyhow::Result<&T> {
        self.storage.require::<T>()
    }

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

    pub(crate) fn unregister_event(&self, id: EventHookId) -> bool {
        self.event_registry.unregister(id)
    }
}
