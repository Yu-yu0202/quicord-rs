/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use crate::command::{scope::CommandScope, CommandHandler};

/// Metadata describing a registered user context command.
pub struct UserContextCommandMetadata {
    /// The command name shown in Discord.
    pub name: &'static str,
    /// The scope where the command is registered.
    pub scope: CommandScope,
    /// The handler invoked when the command is executed.
    pub run: CommandHandler,
}

/// Distributed slice of all registered user context commands.
#[linkme::distributed_slice]
pub static USER_CONTEXT_COMMANDS: [UserContextCommandMetadata];

/// Metadata describing a registered message context command.
pub struct MessageContextCommandMetadata {
    /// The command name shown in Discord.
    pub name: &'static str,
    /// The scope where the command is registered.
    pub scope: CommandScope,
    /// The handler invoked when the command is executed.
    pub run: CommandHandler,
}

/// Distributed slice of all registered message context commands.
#[linkme::distributed_slice]
pub static MESSAGE_CONTEXT_COMMANDS: [MessageContextCommandMetadata];
