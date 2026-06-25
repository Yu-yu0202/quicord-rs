/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

//! Slash command metadata and option definitions.

pub use twilight_model::application::command::CommandOptionType;

use crate::command::{scope::CommandScope, CommandHandler};

/// Metadata for a single slash command option.
#[derive(Clone, Copy)]
pub struct SlashCommandOptionMetadata {
    /// The option name exposed to Discord.
    pub name: &'static str,
    /// The option description exposed to Discord.
    pub description: &'static str,
    /// The Discord option type.
    pub kind: CommandOptionType,
    /// Whether the option is required.
    pub required: bool,
}

/// Metadata describing a registered slash command.
pub struct SlashCommandMetadata {
    /// The command name shown in Discord.
    pub name: &'static str,
    /// The command description shown in Discord.
    pub description: &'static str,
    /// The scope where the command is registered.
    pub scope: CommandScope,
    /// The command options advertised to Discord.
    pub options: &'static [SlashCommandOptionMetadata],
    /// The handler invoked when the command is executed.
    pub run: CommandHandler,
}

/// Distributed slice of all registered slash commands.
#[linkme::distributed_slice]
pub static SLASH_COMMANDS: [SlashCommandMetadata];
