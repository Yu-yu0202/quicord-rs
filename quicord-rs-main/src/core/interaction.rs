/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use crate::core::client::Client;
use anyhow::Result;
use twilight_model::application::interaction::modal::ModalInteractionComponent;
pub use twilight_model::channel::message::component::ButtonStyle;
use twilight_model::{
    application::interaction::{
        application_command::{CommandData, CommandDataOption, CommandOptionValue}, message_component::MessageComponentInteractionData,
        modal::ModalInteractionData,
        Interaction,
        InteractionData,
    },
    channel::{message::MessageFlags, Channel, Message},
    gateway::event::Event,
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
    id::{
        marker::{
            AttachmentMarker, ChannelMarker, GenericMarker, GuildMarker, InteractionMarker,
            RoleMarker, UserMarker,
        },
        Id,
    },
    user::User,
};
pub use twilight_util::builder::embed::EmbedBuilder;
pub use twilight_util::builder::message::{
    ActionRowBuilder, ButtonBuilder, ContainerBuilder, SelectMenuBuilder, SelectMenuOptionBuilder,
    TextDisplayBuilder,
};
pub use twilight_util::builder::InteractionResponseDataBuilder as InteractionResponseBuilder;

/// Converts a value into a Discord interaction response payload.
pub trait IntoResponse {
    /// Builds the response payload.
    fn into_response(self) -> InteractionResponseData;
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

/// A view for accessing the options of an interaction.
pub struct CommandOptionsView<'a> {
    data: &'a [CommandDataOption],
}

impl<'a> CommandOptionsView<'a> {
    pub(crate) fn new(data: &'a [CommandDataOption]) -> Self {
        Self { data }
    }

    /// Returns a string option value by name.
    pub fn string(&self, name: &str) -> Option<&'a str> {
        self.data
            .iter()
            .find(|option| option.name == name)
            .and_then(|option| match &option.value {
                CommandOptionValue::String(value) => Some(value.as_str()),
                _ => None,
            })
    }

    /// Returns an integer option value by name.
    pub fn integer(&self, name: &str) -> Option<i64> {
        self.data
            .iter()
            .find(|option| option.name == name)
            .and_then(|option| match &option.value {
                CommandOptionValue::Integer(value) => Some(*value),
                _ => None,
            })
    }

    /// Returns a number option value by name.
    pub fn number(&self, name: &str) -> Option<f64> {
        self.data
            .iter()
            .find(|option| option.name == name)
            .and_then(|option| match &option.value {
                CommandOptionValue::Number(value) => Some(*value),
                _ => None,
            })
    }

    /// Returns a boolean option value by name.
    pub fn boolean(&self, name: &str) -> Option<bool> {
        self.data
            .iter()
            .find(|option| option.name == name)
            .and_then(|option| match &option.value {
                CommandOptionValue::Boolean(value) => Some(*value),
                _ => None,
            })
    }

    /// Returns a user option value by name.
    pub fn user(&self, name: &str) -> Option<Id<UserMarker>> {
        self.data
            .iter()
            .find(|option| option.name == name)
            .and_then(|option| match &option.value {
                CommandOptionValue::User(value) => Some(*value),
                _ => None,
            })
    }

    /// Returns a role option value by name.
    pub fn role(&self, name: &str) -> Option<Id<RoleMarker>> {
        self.data
            .iter()
            .find(|option| option.name == name)
            .and_then(|option| match &option.value {
                CommandOptionValue::Role(value) => Some(*value),
                _ => None,
            })
    }

    /// Returns a channel option value by name.
    pub fn channel(&self, name: &str) -> Option<Id<ChannelMarker>> {
        self.data
            .iter()
            .find(|option| option.name == name)
            .and_then(|option| match &option.value {
                CommandOptionValue::Channel(value) => Some(*value),
                _ => None,
            })
    }

    /// Returns a mentionable option value by name.
    pub fn mentionable(&self, name: &str) -> Option<Id<GenericMarker>> {
        self.data
            .iter()
            .find(|option| option.name == name)
            .and_then(|option| match &option.value {
                CommandOptionValue::Mentionable(value) => Some(*value),
                _ => None,
            })
    }

    /// Returns an attachment option value by name.
    pub fn attachment(&self, name: &str) -> Option<Id<AttachmentMarker>> {
        self.data
            .iter()
            .find(|option| option.name == name)
            .and_then(|option| match &option.value {
                CommandOptionValue::Attachment(value) => Some(*value),
                _ => None,
            })
    }

    /// Returns a subcommand option value by name.
    pub fn subcommand(&self, name: &str) -> Option<SubCommandOptions> {
        self.data
            .iter()
            .find(|option| option.name == name)
            .and_then(|option| match &option.value {
                CommandOptionValue::SubCommand(options) => Some(SubCommandOptions(options.clone())),
                _ => None,
            })
    }

    /// Returns a subcommand group option value by name.
    pub fn subcommand_group(&self, name: &str) -> Option<SubCommandGroupOptions> {
        self.data
            .iter()
            .find(|option| option.name == name)
            .and_then(|option| match &option.value {
                CommandOptionValue::SubCommandGroup(options) => {
                    Some(SubCommandGroupOptions(options.clone()))
                }
                _ => None,
            })
    }
}

pub(crate) trait HasCustomId {
    fn custom_id(&self) -> Option<&str>;
}

impl HasCustomId for ModalInteractionComponent {
    fn custom_id(&self) -> Option<&str> {
        match self {
            Self::TextInput(v) => Some(&v.custom_id),
            Self::StringSelect(v) => Some(&v.custom_id),
            Self::UserSelect(v) => Some(&v.custom_id),
            Self::RoleSelect(v) => Some(&v.custom_id),
            Self::MentionableSelect(v) => Some(&v.custom_id),
            Self::ChannelSelect(v) => Some(&v.custom_id),
            Self::FileUpload(v) => Some(&v.custom_id),

            Self::ActionRow(_) | Self::Label(_) | Self::TextDisplay(_) | Self::Unknown(_) => None,
        }
    }
}

/// A view for accessing the modal input of an interaction.
pub struct ModalView<'a> {
    components: &'a [ModalInteractionComponent],
}

impl<'a> ModalView<'a> {
    pub(crate) fn new(components: &'a [ModalInteractionComponent]) -> Self {
        Self { components }
    }

    /// Returns the component with the given custom ID.
    pub fn component(&self, custom_id: &str) -> Option<&'a ModalInteractionComponent> {
        self.components
            .iter()
            .find(|component| component.custom_id() == Some(custom_id))
    }

    /// Returns the text input value by custom ID.
    pub fn text(&self, custom_id: &str) -> Option<&'a str> {
        self.component(custom_id)
            .and_then(|component| match component {
                ModalInteractionComponent::TextInput(v) => Some(v.value.as_str()),
                _ => None,
            })
    }

    /// Returns the string select values by custom ID.
    pub fn strings(&self, custom_id: &str) -> Option<&'a [String]> {
        self.component(custom_id)
            .and_then(|component| match component {
                ModalInteractionComponent::StringSelect(v) => Some(v.values.as_slice()),
                _ => None,
            })
    }

    /// Returns the user select values by custom ID.
    pub fn users(&self, custom_id: &str) -> Option<&'a [Id<UserMarker>]> {
        self.component(custom_id)
            .and_then(|component| match component {
                ModalInteractionComponent::UserSelect(v) => Some(v.values.as_slice()),
                _ => None,
            })
    }

    /// Returns the role select values by custom ID.
    pub fn roles(&self, custom_id: &str) -> Option<&'a [Id<RoleMarker>]> {
        self.component(custom_id)
            .and_then(|component| match component {
                ModalInteractionComponent::RoleSelect(v) => Some(v.values.as_slice()),
                _ => None,
            })
    }

    /// Returns the mentionable select values by custom ID.
    pub fn mentionables(&self, custom_id: &str) -> Option<&'a [Id<GenericMarker>]> {
        self.component(custom_id)
            .and_then(|component| match component {
                ModalInteractionComponent::MentionableSelect(v) => Some(v.values.as_slice()),
                _ => None,
            })
    }

    /// Returns the channel select values by custom ID.
    pub fn channels(&self, custom_id: &str) -> Option<&'a [Id<ChannelMarker>]> {
        self.component(custom_id)
            .and_then(|component| match component {
                ModalInteractionComponent::ChannelSelect(v) => Some(v.values.as_slice()),
                _ => None,
            })
    }

    /// Returns the file upload values by custom ID.
    pub fn attachments(&self, custom_id: &str) -> Option<&'a [Id<AttachmentMarker>]> {
        self.component(custom_id)
            .and_then(|component| match component {
                ModalInteractionComponent::FileUpload(v) => Some(v.values.as_slice()),
                _ => None,
            })
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

    /// Returns options view if the interaction is a slash command.
    pub fn options(&self) -> Option<CommandOptionsView<'_>> {
        self.command_data()
            .map(|data| CommandOptionsView::new(data.options.as_slice()))
    }

    /// Returns a modal input view if the interaction is a modal submit.
    pub fn modal(&self) -> Option<ModalView<'_>> {
        self.modal_data()
            .map(|data| ModalView::new(data.components.as_slice()))
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
