pub use twilight_model::application::command::CommandOptionType;

use crate::command::{CommandHandler, scope::CommandScope};

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
