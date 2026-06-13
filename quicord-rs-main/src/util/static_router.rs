/*
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use rustc_hash::FxBuildHasher;
use std::{borrow::Borrow, collections::HashMap, hash::Hash};

pub type FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;

pub struct StaticRouter<K: 'static + Hash + Eq, V: 'static> {
    table: FxHashMap<K, &'static V>,
}

impl<K: 'static + Hash + Eq, V: 'static> StaticRouter<K, V> {
    pub fn new<I>(items: I, key_extractor: fn(&'static V) -> K) -> Self
    where
        I: IntoIterator<Item = &'static V>,
    {
        let mut table = FxHashMap::<K, &'static V>::default();

        for item in items {
            let key = key_extractor(item);
            table.insert(key, item);
        }

        Self { table }
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&'static V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.table.get(key).copied()
    }
}
