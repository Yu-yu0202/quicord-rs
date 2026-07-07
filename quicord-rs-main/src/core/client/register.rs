/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
use crate::command::context::{MESSAGE_CONTEXT_COMMANDS, USER_CONTEXT_COMMANDS};
use crate::command::scope::CommandScope;
use crate::command::slash::SLASH_COMMANDS;
use crate::Bot;
use rustc_hash::{FxBuildHasher, FxHashMap};
use tracing::info;
use twilight_model::application::command::{Command, CommandOption, CommandType};
use twilight_util::builder::command::CommandBuilder;

/// Guild ID to command list mapping used while registering commands.
type GuildCommandMap = FxHashMap<&'static str, Vec<Command>>;

/// Pending command registrations grouped by scope.
pub struct PendingCommands {
    pub(crate) global: Vec<Command>,
    pub(crate) guild: GuildCommandMap,
}

impl PendingCommands {
    /// Creates an empty command collection.
    pub(crate) fn new() -> Self {
        Self {
            global: Vec::new(),
            guild: GuildCommandMap::with_hasher(FxBuildHasher::default()),
        }
    }

    /// Returns whether no commands are queued for registration.
    pub(crate) fn is_empty(&self) -> bool {
        self.global.is_empty() && self.guild.is_empty()
    }

    /// Adds a command to the appropriate scope buckets.
    pub(crate) fn push(&mut self, scope: CommandScope, command: Command) {
        match scope {
            CommandScope::Global => self.global.push(command),
            CommandScope::Guild(guild_ids) => {
                for guild_id in guild_ids {
                    self.guild
                        .entry(*guild_id)
                        .or_insert_with(Vec::new)
                        .push(command.clone());
                }
            }
        }
    }
}

impl Bot {
    /// Builds and uploads all registered commands to Discord.
    pub(crate) async fn register_commands(&self) -> anyhow::Result<()> {
        let mut commands = PendingCommands::new();

        for slash in SLASH_COMMANDS.iter() {
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

            let command = builder.build();

            commands.push(slash.scope, command);
        }

        for user in USER_CONTEXT_COMMANDS.iter() {
            let command = CommandBuilder::new(user.name, "", CommandType::User).build();

            commands.push(user.scope, command);
        }

        for message in MESSAGE_CONTEXT_COMMANDS.iter() {
            let command = CommandBuilder::new(message.name, "", CommandType::Message).build();

            commands.push(message.scope, command);
        }

        if commands.is_empty() {
            return Ok(());
        }

        let interaction_client = self.client.http.interaction(self.application_id);

        if !commands.global.is_empty() {
            interaction_client
                .set_global_commands(&commands.global)
                .await?;

            info!("Registered {} global commands", commands.global.len());
        }

        for (guild_id, commands) in commands.guild {
            interaction_client
                .set_guild_commands(guild_id.parse()?, &commands)
                .await?;

            info!(
                "Registered {} commands for guild {}",
                commands.len(),
                guild_id
            );
        }

        Ok(())
    }
}
