/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use crate::Bot;
use crate::command::context::{
    MESSAGE_CONTEXT_COMMANDS, MessageContextCommandMetadata, USER_CONTEXT_COMMANDS,
    UserContextCommandMetadata,
};
use crate::command::scope::CommandScope;
use crate::command::slash::{SLASH_COMMANDS, SlashCommandMetadata};
use rustc_hash::{FxBuildHasher, FxHashMap};
use tracing::info;
use twilight_model::application::command::{Command, CommandOption, CommandType};
use twilight_model::id::{Id, marker::GuildMarker};
use twilight_util::builder::command::CommandBuilder;

/// Guild ID to static command metadata mapping used while registering commands.
type GuildCommandMap = FxHashMap<Id<GuildMarker>, Vec<PlannedCommand>>;

/// Static command metadata selected for one registration target.
#[derive(Clone, Copy)]
enum PlannedCommand {
    Slash(&'static SlashCommandMetadata),
    UserContext(&'static UserContextCommandMetadata),
    MessageContext(&'static MessageContextCommandMetadata),
}

impl PlannedCommand {
    fn build(self) -> Command {
        match self {
            Self::Slash(slash) => {
                let mut builder =
                    CommandBuilder::new(slash.name, slash.description, CommandType::ChatInput);

                for option in slash.options {
                    builder = builder.option(CommandOption {
                        autocomplete: None,
                        channel_types: None,
                        choices: None,
                        description: option.description.to_string(),
                        description_localizations: None,
                        kind: option.kind,
                        max_length: None,
                        max_value: None,
                        min_length: None,
                        min_value: None,
                        name: option.name.to_string(),
                        name_localizations: None,
                        options: None,
                        required: Some(option.required),
                    });
                }

                builder.build()
            }
            Self::UserContext(user) => {
                CommandBuilder::new(user.name, "", CommandType::User).build()
            }
            Self::MessageContext(message) => {
                CommandBuilder::new(message.name, "", CommandType::Message).build()
            }
        }
    }
}

/// Registration plan grouped by scope without cloning command payloads.
struct CommandRegistrationPlan {
    global: Vec<PlannedCommand>,
    guild: FxHashMap<Id<GuildMarker>, Vec<PlannedCommand>>,
}

impl CommandRegistrationPlan {
    /// Creates an empty command collection.
    fn new() -> Self {
        Self {
            global: Vec::new(),
            guild: GuildCommandMap::with_hasher(FxBuildHasher),
        }
    }

    /// Returns whether no commands are queued for registration.
    fn is_empty(&self) -> bool {
        self.global.is_empty() && self.guild.is_empty()
    }

    /// Adds a command to the appropriate scope buckets.
    fn push(&mut self, scope: CommandScope, command: PlannedCommand) {
        match scope {
            CommandScope::Global => self.global.push(command),
            CommandScope::Guild(guild_ids) => {
                for guild_id in guild_ids {
                    self.guild.entry(*guild_id).or_default().push(command);
                }
            }
        }
    }

    fn from_static() -> Self {
        let mut plan = Self::new();

        for slash in SLASH_COMMANDS.iter() {
            plan.push(slash.scope, PlannedCommand::Slash(slash));
        }

        for user in USER_CONTEXT_COMMANDS.iter() {
            plan.push(user.scope, PlannedCommand::UserContext(user));
        }

        for message in MESSAGE_CONTEXT_COMMANDS.iter() {
            plan.push(message.scope, PlannedCommand::MessageContext(message));
        }

        plan
    }

    /// Builds the payload for the global registration request.
    fn global_payload(&self) -> Vec<Command> {
        self.global
            .iter()
            .copied()
            .map(PlannedCommand::build)
            .collect()
    }

    /// Builds the payload for one guild registration request.
    fn guild_payload(&self, guild_id: Id<GuildMarker>) -> Option<Vec<Command>> {
        self.guild.get(&guild_id).map(|commands| {
            commands
                .iter()
                .copied()
                .map(PlannedCommand::build)
                .collect()
        })
    }
}

impl Bot {
    async fn register_global_commands(&self, plan: &CommandRegistrationPlan) -> anyhow::Result<()> {
        let commands = plan.global_payload();
        if commands.is_empty() {
            return Ok(());
        }

        self.client
            .http
            .interaction(self.application_id)
            .set_global_commands(&commands)
            .await?;

        info!("Registered {} global commands", commands.len());
        Ok(())
    }

    async fn register_guild_commands(
        &self,
        plan: &CommandRegistrationPlan,
        guild_id: Id<GuildMarker>,
    ) -> anyhow::Result<()> {
        let Some(commands) = plan.guild_payload(guild_id) else {
            return Ok(());
        };

        self.client
            .http
            .interaction(self.application_id)
            .set_guild_commands(guild_id, &commands)
            .await?;

        info!(
            "Registered {} commands for guild {}",
            commands.len(),
            guild_id
        );
        Ok(())
    }

    /// Builds and uploads all registered commands to Discord.
    pub(crate) async fn register_commands(&self) -> anyhow::Result<()> {
        let commands = CommandRegistrationPlan::from_static();

        if commands.is_empty() {
            return Ok(());
        }

        self.register_global_commands(&commands).await?;

        for guild_id in commands.guild.keys().copied() {
            self.register_guild_commands(&commands, guild_id).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandRegistrationPlan, PlannedCommand};
    use crate::command::CommandFuture;
    use crate::command::scope::CommandScope;
    use crate::command::slash::SlashCommandMetadata;
    use crate::core::interaction::InteractionContext;
    use twilight_model::id::Id;

    fn handler(_: InteractionContext) -> CommandFuture {
        Box::pin(async { Ok(()) })
    }

    static COMMAND: SlashCommandMetadata = SlashCommandMetadata {
        name: "test",
        description: "test command",
        scope: CommandScope::Guild(&[Id::new(1), Id::new(2)]),
        options: &[],
        run: handler,
    };

    #[test]
    fn plan_reuses_metadata_across_guilds() {
        let mut plan = CommandRegistrationPlan::new();
        plan.push(COMMAND.scope, PlannedCommand::Slash(&COMMAND));

        assert_eq!(plan.guild.len(), 2);
        assert!(plan.guild.values().all(|commands| commands.len() == 1));
    }
}
