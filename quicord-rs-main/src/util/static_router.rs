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
    /// Builds a router from static items, rejecting duplicate keys.
    pub fn try_new<I>(items: I, key_extractor: fn(&'static V) -> K) -> Result<Self, K>
    where
        I: IntoIterator<Item = &'static V>,
    {
        let items = items.into_iter();

        let mut table = FxHashMap::<K, &'static V>::with_capacity_and_hasher(
            items.size_hint().0,
            FxBuildHasher,
        );

        for item in items {
            let key = key_extractor(item);
            if table.contains_key(&key) {
                return Err(key);
            }
            table.insert(key, item);
        }

        Ok(Self { table })
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

#[cfg(test)]
mod tests {
    use super::StaticRouter;

    #[test]
    fn rejects_duplicate_keys() {
        static ITEMS: [&str; 2] = ["first", "second"];

        let duplicate = match StaticRouter::try_new(ITEMS.iter(), |_| "same") {
            Ok(_) => panic!("duplicate keys must be rejected"),
            Err(duplicate) => duplicate,
        };

        assert_eq!(duplicate, "same");
    }
}
