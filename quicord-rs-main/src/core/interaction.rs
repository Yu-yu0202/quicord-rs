/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

mod command;
mod component;
mod modal;
mod response;

pub(crate) mod r#trait;

pub mod view;

use crate::core::client::Client;
use crate::core::storage::Storage;
use twilight_model::{
    application::interaction::{Interaction, InteractionData},
    channel::{Channel, Message},
    gateway::event::Event,
    id::{
        Id,
        marker::{ChannelMarker, GuildMarker, InteractionMarker, UserMarker},
    },
    user::User,
};
pub use twilight_util::builder::InteractionResponseDataBuilder as InteractionResponseBuilder;
pub use twilight_util::builder::embed::EmbedBuilder;
pub use twilight_util::builder::message::{
    ActionRowBuilder, ButtonBuilder, ContainerBuilder, SelectMenuBuilder, SelectMenuOptionBuilder,
    TextDisplayBuilder,
};

/// Context passed to interaction handlers.
#[derive(Clone)]
pub struct InteractionContext {
    /// The bot client.
    pub client: Client,
    bot_storage: Storage,
    /// The raw gateway event.
    pub event: Event,
}

impl InteractionContext {
    /// Creates a new interaction context.
    pub(crate) fn new(client: Client, storage: Storage, event: Event) -> Self {
        Self {
            client,
            bot_storage: storage,
            event,
        }
    }

    /// Returns a shared reference to a value from bot storage.
    pub fn storage<T: Send + Sync + 'static>(&self) -> anyhow::Result<&T> {
        self.bot_storage.require::<T>()
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
        self.channel().map(|channel| channel.id)
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
}
