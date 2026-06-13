/*
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use crate::core::client::Client;
use anyhow::Result;
use std::{error::Error, fmt};
use twilight_model::{
    application::{
        command::CommandOptionType,
        interaction::{
            Interaction, InteractionData,
            application_command::{CommandData, CommandDataOption, CommandOptionValue},
            message_component::MessageComponentInteractionData,
            modal::ModalInteractionData,
        },
    },
    channel::{Channel, Message, message::MessageFlags},
    gateway::event::Event,
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
    id::{
        Id,
        marker::{
            AttachmentMarker, ChannelMarker, GenericMarker, GuildMarker, InteractionMarker,
            RoleMarker, UserMarker,
        },
    },
    user::User,
};
pub use twilight_util::builder::InteractionResponseDataBuilder as InteractionResponseBuilder;
pub use twilight_util::builder::embed::EmbedBuilder;

/// Converts a value into a Discord interaction response payload.
pub trait IntoResponse {
    /// Builds the response payload.
    fn into_response(self) -> InteractionResponseData;
}

/// Errors returned when a command option is missing or has the wrong type.
#[derive(Debug)]
pub enum CommandOptionError {
    /// The requested option was not present.
    Missing {
        /// The missing option name.
        name: String,
    },
    /// The option was present but had an unexpected type.
    TypeMismatch {
        /// The option name.
        name: String,
        /// The expected Discord type.
        expected: CommandOptionType,
        /// The actual Discord type.
        actual: CommandOptionType,
    },
}

impl fmt::Display for CommandOptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { name } => write!(f, "missing command option `{name}`"),
            Self::TypeMismatch {
                name,
                expected,
                actual,
            } => write!(
                f,
                "command option `{name}` has type `{actual:?}`, expected `{expected:?}`"
            ),
        }
    }
}

impl Error for CommandOptionError {}

/// Converts a raw Discord option value into a strongly typed value.
pub trait FromCommandOptionValue: Sized {
    /// The Discord option type required by the target value.
    const EXPECTED_TYPE: CommandOptionType;

    /// Attempts to convert the raw option value.
    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self>;
}

/// Wrapper around the options for a subcommand.
#[derive(Clone, Debug, PartialEq)]
pub struct SubCommandOptions(Vec<CommandDataOption>);

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

/// Wrapper around the options for a subcommand group.
#[derive(Clone, Debug, PartialEq)]
pub struct SubCommandGroupOptions(Vec<CommandDataOption>);

impl SubCommandGroupOptions {
    /// Returns the options as a slice.
    pub fn as_slice(&self) -> &[CommandDataOption] {
        &self.0
    }

    /// Consumes the wrapper and returns the underlying vector.
    pub fn into_inner(self) -> Vec<CommandDataOption> {
        self.0
    }
}

impl AsRef<[CommandDataOption]> for SubCommandGroupOptions {
    fn as_ref(&self) -> &[CommandDataOption] {
        self.as_slice()
    }
}

/// Converts a string slice into a plain text response.
impl IntoResponse for &str {
    fn into_response(self) -> InteractionResponseData {
        InteractionResponseData {
            content: Some(self.to_string()),
            ..Default::default()
        }
    }
}

/// Converts an owned string into a plain text response.
impl IntoResponse for String {
    fn into_response(self) -> InteractionResponseData {
        InteractionResponseData {
            content: Some(self),
            ..Default::default()
        }
    }
}

/// Returns an interaction response builder unchanged.
impl IntoResponse for InteractionResponseBuilder {
    fn into_response(self) -> InteractionResponseData {
        self.build()
    }
}

/// Returns an already built interaction response unchanged.
impl IntoResponse for InteractionResponseData {
    fn into_response(self) -> InteractionResponseData {
        self
    }
}

/// Context passed to interaction handlers.
#[derive(Clone)]
pub struct InteractionContext {
    /// The bot client.
    pub client: Client,
    /// The raw gateway event.
    pub event: Event,
}

impl InteractionContext {
    /// Creates a new interaction context.
    pub(crate) fn new(client: Client, event: Event) -> Self {
        Self { client, event }
    }

    /// Sends a channel message response for the interaction.
    pub async fn reply(&self, response: impl IntoResponse) -> Result<()> {
        if self.interaction().is_some() {
            let data = response.into_response();

            self.create_response(
                InteractionResponseType::ChannelMessageWithSource,
                Some(data),
            )
            .await?;
        }

        Ok(())
    }

    /// Defers the initial response and optionally marks it ephemeral.
    pub async fn defer_reply(&self, ephemeral: bool) -> Result<()> {
        if self.interaction().is_some() {
            self.create_response(
                InteractionResponseType::DeferredChannelMessageWithSource,
                ephemeral.then(ephemeral_response_data),
            )
            .await?;
        }

        Ok(())
    }

    /// Edits the original response message.
    pub async fn edit_reply(&self, response: impl IntoResponse) -> Result<()> {
        if let Some(interaction) = self.interaction() {
            let data = response.into_response();
            let json = serde_json::to_vec(&data)?;

            self.client
                .http
                .interaction(interaction.application_id)
                .update_response(&interaction.token)
                .payload_json(&json)
                .await?;
        }

        Ok(())
    }

    /// Returns the underlying interaction if the event is an interaction create.
    pub fn interaction(&self) -> Option<&Interaction> {
        match &self.event {
            Event::InteractionCreate(interaction) => Some(interaction),
            _ => None,
        }
    }

    /// Returns the interaction ID if available.
    pub fn interaction_id(&self) -> Option<Id<InteractionMarker>> {
        self.interaction().map(|interaction| interaction.id)
    }

    /// Returns the interaction author if available.
    pub fn author(&self) -> Option<&User> {
        self.interaction().and_then(Interaction::author)
    }

    /// Returns the author ID if available.
    pub fn author_id(&self) -> Option<Id<UserMarker>> {
        self.interaction().and_then(Interaction::author_id)
    }

    /// Returns the guild ID if the interaction occurred in a guild.
    pub fn guild_id(&self) -> Option<Id<GuildMarker>> {
        self.interaction()
            .and_then(|interaction| interaction.guild_id)
    }

    /// Returns the cached channel object if available.
    pub fn channel(&self) -> Option<&Channel> {
        self.interaction()
            .and_then(|interaction| interaction.channel.as_ref())
    }

    /// Returns the channel ID if available.
    pub fn channel_id(&self) -> Option<Id<ChannelMarker>> {
        if let Some(channel) = self.channel() {
            Some(channel.id)
        } else {
            None
        }
    }

    /// Returns the message attached to the interaction, if any.
    pub fn message(&self) -> Option<&Message> {
        self.interaction()
            .and_then(|interaction| interaction.message.as_ref())
    }

    /// Returns the raw interaction data, if any.
    pub fn data(&self) -> Option<&InteractionData> {
        self.interaction()
            .and_then(|interaction| interaction.data.as_ref())
    }

    /// Returns slash command data for application command interactions.
    pub fn command_data(&self) -> Option<&CommandData> {
        match self.data()? {
            InteractionData::ApplicationCommand(data) => Some(data.as_ref()),
            _ => None,
        }
    }

    /// Returns message component data for component interactions.
    pub fn component_data(&self) -> Option<&MessageComponentInteractionData> {
        match self.data()? {
            InteractionData::MessageComponent(data) => Some(data.as_ref()),
            _ => None,
        }
    }

    /// Returns modal submit data for modal interactions.
    pub fn modal_data(&self) -> Option<&ModalInteractionData> {
        match self.data()? {
            InteractionData::ModalSubmit(data) => Some(data.as_ref()),
            _ => None,
        }
    }

    /// Returns the slash or context command name if available.
    pub fn command_name(&self) -> Option<&str> {
        self.command_data().map(|data| data.name.as_str())
    }

    /// Returns a single command option by name.
    pub fn command_option(&self, name: &str) -> Option<&CommandDataOption> {
        self.command_data()?
            .options
            .iter()
            .find(|option| option.name == name)
    }

    /// Reads a command option and converts it into the requested type.
    pub fn option<T>(&self, name: &str) -> std::result::Result<Option<T>, CommandOptionError>
    where
        T: FromCommandOptionValue,
    {
        let Some(option) = self.command_option(name) else {
            return Ok(None);
        };

        T::from_command_option_value(&option.value)
            .map(Some)
            .ok_or_else(|| CommandOptionError::TypeMismatch {
                name: name.to_string(),
                expected: T::EXPECTED_TYPE,
                actual: option.value.kind(),
            })
    }

    /// Reads a required command option and converts it into the requested type.
    pub fn required_option<T>(&self, name: &str) -> std::result::Result<T, CommandOptionError>
    where
        T: FromCommandOptionValue,
    {
        self.option(name)?
            .ok_or_else(|| CommandOptionError::Missing {
                name: name.to_string(),
            })
    }

    /// Sends a raw interaction response to Discord.
    async fn create_response(
        &self,
        kind: InteractionResponseType,
        data: Option<InteractionResponseData>,
    ) -> Result<()> {
        if let Some(interaction) = self.interaction() {
            let payload = InteractionResponse { kind, data };

            self.client
                .http
                .interaction(interaction.application_id)
                .create_response(interaction.id, &interaction.token, &payload)
                .await?;
        }

        Ok(())
    }
}

/// Builds the payload used for ephemeral interaction responses.
fn ephemeral_response_data() -> InteractionResponseData {
    InteractionResponseData {
        flags: Some(MessageFlags::EPHEMERAL),
        ..Default::default()
    }
}

/// Converts attachment option values into attachment IDs.
impl FromCommandOptionValue for Id<AttachmentMarker> {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::Attachment;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::Attachment(value) => Some(*value),
            _ => None,
        }
    }
}

/// Converts boolean option values into `bool`.
impl FromCommandOptionValue for bool {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::Boolean;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::Boolean(value) => Some(*value),
            _ => None,
        }
    }
}

/// Converts channel option values into channel IDs.
impl FromCommandOptionValue for Id<ChannelMarker> {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::Channel;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::Channel(value) => Some(*value),
            _ => None,
        }
    }
}

/// Converts integer option values into `i64`.
impl FromCommandOptionValue for i64 {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::Integer;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::Integer(value) => Some(*value),
            _ => None,
        }
    }
}

/// Converts mentionable option values into generic IDs.
impl FromCommandOptionValue for Id<GenericMarker> {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::Mentionable;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::Mentionable(value) => Some(*value),
            _ => None,
        }
    }
}

/// Converts number option values into `f64`.
impl FromCommandOptionValue for f64 {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::Number;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::Number(value) => Some(*value),
            _ => None,
        }
    }
}

/// Converts role option values into role IDs.
impl FromCommandOptionValue for Id<RoleMarker> {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::Role;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::Role(value) => Some(*value),
            _ => None,
        }
    }
}

/// Converts string option values into owned strings.
impl FromCommandOptionValue for String {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::String;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::String(value) => Some(value.clone()),
            _ => None,
        }
    }
}

/// Converts subcommand option values into `SubCommandOptions`.
impl FromCommandOptionValue for SubCommandOptions {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::SubCommand;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::SubCommand(value) => Some(Self(value.clone())),
            _ => None,
        }
    }
}

/// Converts subcommand group option values into `SubCommandGroupOptions`.
impl FromCommandOptionValue for SubCommandGroupOptions {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::SubCommandGroup;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::SubCommandGroup(value) => Some(Self(value.clone())),
            _ => None,
        }
    }
}

/// Converts user option values into user IDs.
impl FromCommandOptionValue for Id<UserMarker> {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::User;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::User(value) => Some(*value),
            _ => None,
        }
    }
}
