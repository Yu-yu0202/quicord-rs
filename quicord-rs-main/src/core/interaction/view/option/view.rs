/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use crate::core::interaction::view::option::SubCommandGroupOptions;
use crate::core::interaction::view::option::SubCommandOptions;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue,
};
use twilight_model::application::interaction::modal::ModalInteractionComponent;
use twilight_model::id::marker::{
    AttachmentMarker, ChannelMarker, GenericMarker, RoleMarker, UserMarker,
};
use twilight_model::id::Id;

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

pub trait HasCustomId {
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
