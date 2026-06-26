/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use crate::command::CommandHandler;

/// Metadata describing a registered button.
pub struct ButtonMetadata {
    /// The button custom ID in Discord.
    pub custom_id: &'static str,
    /// The handler invoked when the button is clicked.
    pub run: CommandHandler,
}

/// Distributed slice of all registered buttons.
#[linkme::distributed_slice]
pub static BUTTONS: [ButtonMetadata];

/// Metadata describing a registered select menu.
pub struct SelectMenuMetadata {
    /// The select menu custom ID in Discord.
    pub custom_id: &'static str,
    /// The handler invoked when the select menu is used.
    pub run: CommandHandler,
}

/// Distributed slice of all registered select menus.
#[linkme::distributed_slice]
pub static SELECT_MENUS: [SelectMenuMetadata];
