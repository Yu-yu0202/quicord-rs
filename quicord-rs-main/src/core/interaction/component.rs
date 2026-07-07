/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
use crate::InteractionContext;
use twilight_model::application::interaction::message_component::MessageComponentInteractionData;
use twilight_model::application::interaction::InteractionData;

impl InteractionContext {
    /// Returns message component data for component interactions.
    pub fn component_data(&self) -> Option<&MessageComponentInteractionData> {
        match self.data()? {
            InteractionData::MessageComponent(data) => Some(data.as_ref()),
            _ => None,
        }
    }
}
