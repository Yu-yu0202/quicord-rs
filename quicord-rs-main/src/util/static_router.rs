/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use rustc_hash::FxBuildHasher;
use std::{borrow::Borrow, collections::HashMap, hash::Hash};

/// Hash map alias using `FxBuildHasher`.
pub type FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;

/// A static lookup table from keys to leaked metadata.
pub struct StaticRouter<K: 'static + Hash + Eq, V: 'static> {
    table: FxHashMap<K, &'static V>,
}

impl<K: 'static + Hash + Eq, V: 'static> StaticRouter<K, V> {
    /// Builds a router from a set of static items and a key extractor.
    pub fn new<I>(items: I, key_extractor: fn(&'static V) -> K) -> Self
    where
        I: IntoIterator<Item = &'static V>,
    {
        let items = items.into_iter();

        let mut table = FxHashMap::<K, &'static V>::with_capacity_and_hasher(
            items.size_hint().0,
            FxBuildHasher::default(),
        );

        for item in items {
            let key = key_extractor(item);
            table.insert(key, item);
        }

        Self { table }
    }

    /// Looks up an item by key.
    pub fn get<Q>(&self, key: &Q) -> Option<&'static V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.table.get(key).copied()
    }
}
