/*
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

pub use twilight_model::application::command::CommandOptionType;

use crate::command::{scope::CommandScope, CommandHandler};

#[derive(Clone, Copy)]
pub struct SlashCommandOptionMetadata {
    pub name: &'static str,
    pub description: &'static str,
    pub kind: CommandOptionType,
    pub required: bool,
}

pub struct SlashCommandMetadata {
    pub name: &'static str,
    pub description: &'static str,
    pub scope: CommandScope,
    pub options: &'static [SlashCommandOptionMetadata],
    pub run: CommandHandler,
}

#[linkme::distributed_slice]
pub static SLASH_COMMANDS: [SlashCommandMetadata];
