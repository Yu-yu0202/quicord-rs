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

pub trait IntoResponse {
    fn into_response(self) -> InteractionResponseData;
}

#[derive(Debug)]
pub enum CommandOptionError {
    Missing {
        name: String,
    },
    TypeMismatch {
        name: String,
        expected: CommandOptionType,
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

pub trait FromCommandOptionValue: Sized {
    const EXPECTED_TYPE: CommandOptionType;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubCommandOptions(Vec<CommandDataOption>);

impl SubCommandOptions {
    pub fn as_slice(&self) -> &[CommandDataOption] {
        &self.0
    }

    pub fn into_inner(self) -> Vec<CommandDataOption> {
        self.0
    }
}

impl AsRef<[CommandDataOption]> for SubCommandOptions {
    fn as_ref(&self) -> &[CommandDataOption] {
        self.as_slice()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubCommandGroupOptions(Vec<CommandDataOption>);

impl SubCommandGroupOptions {
    pub fn as_slice(&self) -> &[CommandDataOption] {
        &self.0
    }

    pub fn into_inner(self) -> Vec<CommandDataOption> {
        self.0
    }
}

impl AsRef<[CommandDataOption]> for SubCommandGroupOptions {
    fn as_ref(&self) -> &[CommandDataOption] {
        self.as_slice()
    }
}

impl IntoResponse for &str {
    fn into_response(self) -> InteractionResponseData {
        InteractionResponseData {
            content: Some(self.to_string()),
            ..Default::default()
        }
    }
}

impl IntoResponse for String {
    fn into_response(self) -> InteractionResponseData {
        InteractionResponseData {
            content: Some(self),
            ..Default::default()
        }
    }
}

impl IntoResponse for InteractionResponseBuilder {
    fn into_response(self) -> InteractionResponseData {
        self.build()
    }
}

impl IntoResponse for InteractionResponseData {
    fn into_response(self) -> InteractionResponseData {
        self
    }
}

#[derive(Clone)]
pub struct InteractionContext {
    pub client: Client,
    pub event: Event,
}

impl InteractionContext {
    pub(crate) fn new(client: Client, event: Event) -> Self {
        Self { client, event }
    }

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

    pub fn interaction(&self) -> Option<&Interaction> {
        match &self.event {
            Event::InteractionCreate(interaction) => Some(interaction),
            _ => None,
        }
    }

    pub fn interaction_id(&self) -> Option<Id<InteractionMarker>> {
        self.interaction().map(|interaction| interaction.id)
    }

    pub fn author(&self) -> Option<&User> {
        self.interaction().and_then(Interaction::author)
    }

    pub fn author_id(&self) -> Option<Id<UserMarker>> {
        self.interaction().and_then(Interaction::author_id)
    }

    pub fn guild_id(&self) -> Option<Id<GuildMarker>> {
        self.interaction()
            .and_then(|interaction| interaction.guild_id)
    }

    pub fn channel(&self) -> Option<&Channel> {
        self.interaction()
            .and_then(|interaction| interaction.channel.as_ref())
    }

    pub fn channel_id(&self) -> Option<Id<ChannelMarker>> {
        if let Some(channel) = self.channel() {
            Some(channel.id)
        } else {
            None
        }
    }

    pub fn message(&self) -> Option<&Message> {
        self.interaction()
            .and_then(|interaction| interaction.message.as_ref())
    }

    pub fn data(&self) -> Option<&InteractionData> {
        self.interaction()
            .and_then(|interaction| interaction.data.as_ref())
    }

    pub fn command_data(&self) -> Option<&CommandData> {
        match self.data()? {
            InteractionData::ApplicationCommand(data) => Some(data.as_ref()),
            _ => None,
        }
    }

    pub fn component_data(&self) -> Option<&MessageComponentInteractionData> {
        match self.data()? {
            InteractionData::MessageComponent(data) => Some(data.as_ref()),
            _ => None,
        }
    }

    pub fn modal_data(&self) -> Option<&ModalInteractionData> {
        match self.data()? {
            InteractionData::ModalSubmit(data) => Some(data.as_ref()),
            _ => None,
        }
    }

    pub fn command_name(&self) -> Option<&str> {
        self.command_data().map(|data| data.name.as_str())
    }

    pub fn command_option(&self, name: &str) -> Option<&CommandDataOption> {
        self.command_data()?
            .options
            .iter()
            .find(|option| option.name == name)
    }

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

    pub fn required_option<T>(&self, name: &str) -> std::result::Result<T, CommandOptionError>
    where
        T: FromCommandOptionValue,
    {
        self.option(name)?
            .ok_or_else(|| CommandOptionError::Missing {
                name: name.to_string(),
            })
    }

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

fn ephemeral_response_data() -> InteractionResponseData {
    InteractionResponseData {
        flags: Some(MessageFlags::EPHEMERAL),
        ..Default::default()
    }
}

impl FromCommandOptionValue for Id<AttachmentMarker> {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::Attachment;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::Attachment(value) => Some(*value),
            _ => None,
        }
    }
}

impl FromCommandOptionValue for bool {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::Boolean;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::Boolean(value) => Some(*value),
            _ => None,
        }
    }
}

impl FromCommandOptionValue for Id<ChannelMarker> {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::Channel;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::Channel(value) => Some(*value),
            _ => None,
        }
    }
}

impl FromCommandOptionValue for i64 {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::Integer;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::Integer(value) => Some(*value),
            _ => None,
        }
    }
}

impl FromCommandOptionValue for Id<GenericMarker> {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::Mentionable;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::Mentionable(value) => Some(*value),
            _ => None,
        }
    }
}

impl FromCommandOptionValue for f64 {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::Number;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::Number(value) => Some(*value),
            _ => None,
        }
    }
}

impl FromCommandOptionValue for Id<RoleMarker> {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::Role;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::Role(value) => Some(*value),
            _ => None,
        }
    }
}

impl FromCommandOptionValue for String {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::String;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::String(value) => Some(value.clone()),
            _ => None,
        }
    }
}

impl FromCommandOptionValue for SubCommandOptions {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::SubCommand;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::SubCommand(value) => Some(Self(value.clone())),
            _ => None,
        }
    }
}

impl FromCommandOptionValue for SubCommandGroupOptions {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::SubCommandGroup;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::SubCommandGroup(value) => Some(Self(value.clone())),
            _ => None,
        }
    }
}

impl FromCommandOptionValue for Id<UserMarker> {
    const EXPECTED_TYPE: CommandOptionType = CommandOptionType::User;

    fn from_command_option_value(value: &CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::User(value) => Some(*value),
            _ => None,
        }
    }
}
