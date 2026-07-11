/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;

/// Shared application data registered at bot startup and passed to handlers.
#[derive(Clone, Default)]
pub struct Storage {
    inner: Arc<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl Storage {
    pub(crate) fn new(values: HashMap<TypeId, Box<dyn Any + Send + Sync>>) -> Self {
        Self {
            inner: Arc::new(values),
        }
    }

    pub(crate) fn insert_value<T: Send + Sync + 'static>(
        values: &mut HashMap<TypeId, Box<dyn Any + Send + Sync>>,
        value: T,
    ) -> Option<T> {
        values
            .insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|boxed| boxed.downcast::<T>().ok().map(|value| *value))
    }

    /// Returns a shared reference to a stored value when present.
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref())
    }

    /// Returns a shared reference to a stored value or an error when it is missing.
    pub fn require<T: Send + Sync + 'static>(&self) -> Result<&T> {
        self.get::<T>().ok_or_else(|| {
            anyhow::anyhow!(
                "storage value of type `{}` is not registered",
                std::any::type_name::<T>()
            )
        })
    }
}
