/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
use crate::core::interaction::view::CommandOptionsView;
use crate::InteractionContext;
use twilight_model::application::interaction::application_command::CommandData;
use twilight_model::application::interaction::InteractionData;

impl InteractionContext {
    /// Returns slash command data for application command interactions.
    pub fn command_data(&self) -> Option<&CommandData> {
        match self.data()? {
            InteractionData::ApplicationCommand(data) => Some(data.as_ref()),
            _ => None,
        }
    }

    /// Returns the slash or context command name if available.
    pub fn command_name(&self) -> Option<&str> {
        self.command_data().map(|data| data.name.as_str())
    }

    /// Returns options view if the interaction is a slash command.
    pub fn options(&self) -> Option<CommandOptionsView<'_>> {
        self.command_data()
            .map(|data| CommandOptionsView::new(data.options.as_slice()))
    }
}
