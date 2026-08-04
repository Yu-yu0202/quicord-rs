/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

pub mod button;
pub mod color;
pub mod componentsv2;
pub mod embed;
pub mod modal;
pub mod response;
pub mod text_input;

pub(crate) fn set_items<T>(slot: &mut Option<Vec<T>>, items: impl IntoIterator<Item = T>) {
    *slot = Some(items.into_iter().collect());
}

pub(crate) fn push_item<T>(slot: &mut Option<Vec<T>>, item: T) {
    slot.get_or_insert_with(Vec::new).push(item);
}

pub(crate) fn push_items<T>(slot: &mut Option<Vec<T>>, items: impl IntoIterator<Item = T>) {
    slot.get_or_insert_with(Vec::new).extend(items);
}
