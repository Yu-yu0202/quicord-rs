/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use twilight_model::application::interaction::application_command::CommandDataOption;

/// Wrapper around the options for a subcommand.
#[derive(Clone, Debug, PartialEq)]
pub struct SubCommandOptions(pub(crate) Vec<CommandDataOption>);

impl SubCommandOptions {
    /// Returns the options as a slice.
    pub fn as_slice(&self) -> &[CommandDataOption] {
        &self.0
    }

    /// Consumes the wrapper and returns the underlying vector.
    pub fn into_inner(self) -> Vec<CommandDataOption> {
        self.0
    }
}

impl AsRef<[CommandDataOption]> for SubCommandOptions {
    fn as_ref(&self) -> &[CommandDataOption] {
        self.as_slice()
    }
}
