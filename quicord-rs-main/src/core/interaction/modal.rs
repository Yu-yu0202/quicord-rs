/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
use crate::core::interaction::view::ModalView;
use crate::InteractionContext;
use twilight_model::application::interaction::modal::ModalInteractionData;
use twilight_model::application::interaction::InteractionData;

impl InteractionContext {
    /// Returns modal submit data for modal interactions.
    pub fn modal_data(&self) -> Option<&ModalInteractionData> {
        match self.data()? {
            InteractionData::ModalSubmit(data) => Some(data.as_ref()),
            _ => None,
        }
    }

    /// Returns a modal input view if the interaction is a modal submit.
    pub fn modal(&self) -> Option<ModalView<'_>> {
        self.modal_data()
            .map(|data| ModalView::new(data.components.as_slice()))
    }
}
