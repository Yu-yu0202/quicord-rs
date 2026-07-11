/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
use crate::command::message_component::{ButtonMetadata, SelectMenuMetadata};
use crate::command::modal::ModalMetadata;
use crate::{
    command::{
        context::{
            MessageContextCommandMetadata, UserContextCommandMetadata, MESSAGE_CONTEXT_COMMANDS,
            USER_CONTEXT_COMMANDS,
        },
        slash::{SlashCommandMetadata, SLASH_COMMANDS},
    },
    core::event::{EventHandlerMetadata, EVENT_HANDLERS},
    core::storage::Storage,
    util::static_router::StaticRouter,
};
use anyhow::Result;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use twilight_gateway::{ConfigBuilder, Intents, Shard};
use twilight_http::Client as HttpClient;
use twilight_model::{
    gateway::ShardId,
    id::{marker::ApplicationMarker, Id},
};

mod dispatch;
mod register;
mod router;
mod runtime;
mod tls;

/// Shared HTTP client wrapper used by the bot runtime.
#[derive(Clone)]
pub struct Client {
    /// The underlying Twilight HTTP client.
    pub http: Arc<HttpClient>,
}

/// Builder used to construct a [`Bot`].
pub struct BotBuilder {
    token: String,
    storage: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

/// Fluent builder for registering shared [`Storage`] values before building a [`Bot`].
pub struct BotBuilderStorage {
    builder: BotBuilder,
}

/// Bot runtime state and routing tables.
pub struct Bot {
    /// Shared client access for handlers.
    pub client: Client,
    /// The application ID resolved from Discord.
    pub application_id: Id<ApplicationMarker>,
    pub(crate) shard: Shard,
    pub(crate) storage: Storage,

    event_router: StaticRouter<&'static str, EventHandlerMetadata>,
    slash_router: StaticRouter<&'static str, SlashCommandMetadata>,
    user_context_router: StaticRouter<&'static str, UserContextCommandMetadata>,
    message_context_router: StaticRouter<&'static str, MessageContextCommandMetadata>,
    modal_router: StaticRouter<&'static str, ModalMetadata>,
    button_router: StaticRouter<&'static str, ButtonMetadata>,
    select_menu_router: StaticRouter<&'static str, SelectMenuMetadata>,
}

impl Client {
    /// Creates a new client wrapper from an HTTP client.
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

impl BotBuilder {
    /// Creates a new builder using the provided bot token.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            storage: HashMap::new(),
        }
    }

    /// Returns a fluent builder for registering shared storage values.
    pub fn storage(self) -> BotBuilderStorage {
        BotBuilderStorage { builder: self }
    }

    /// Clones the builder state into a fresh builder.
    pub fn share(self) -> BotBuilder {
        BotBuilder {
            token: self.token,
            storage: HashMap::new(),
        }
    }

    /// Builds a bot by connecting to Discord and preparing routers.
    pub async fn build(self) -> Result<Bot> {
        tls::init_rustls();

        let http = Arc::new(HttpClient::new(self.token.clone()));

        let intents = Intents::all(); // ToDo: Add configuration for intents
        let config = ConfigBuilder::new(self.token, intents).build();
        let shard_id = ShardId::ONE; // ToDo: Add configuration for shard count and id
        let shard = Shard::with_config(shard_id, config);

        let application_id = http.current_user_application().await?.model().await?.id;

        let event_router = StaticRouter::new(EVENT_HANDLERS.iter(), |metadata| metadata.event_type);
        let slash_router = StaticRouter::new(SLASH_COMMANDS.iter(), |metadata| metadata.name);
        let user_context_router =
            StaticRouter::new(USER_CONTEXT_COMMANDS.iter(), |metadata| metadata.name);
        let message_context_router =
            StaticRouter::new(MESSAGE_CONTEXT_COMMANDS.iter(), |metadata| metadata.name);
        let modal_router = StaticRouter::new(crate::command::modal::MODALS.iter(), |metadata| {
            metadata.custom_id
        });
        let button_router = StaticRouter::new(
            crate::command::message_component::BUTTONS.iter(),
            |metadata| metadata.custom_id,
        );
        let select_menu_router = StaticRouter::new(
            crate::command::message_component::SELECT_MENUS.iter(),
            |metadata| metadata.custom_id,
        );

        Ok(Bot {
            client: Client::new(http),
            shard,
            application_id,
            storage: Storage::new(self.storage),
            event_router,
            slash_router,
            user_context_router,
            message_context_router,
            modal_router,
            button_router,
            select_menu_router,
        })
    }
}

impl Bot {
    /// Constructs a bot from the provided builder.
    pub async fn new(config: BotBuilder) -> Result<Self> {
        Ok(config.build().await?)
    }
}

impl BotBuilderStorage {
    /// Inserts a value into the bot's shared storage.
    pub fn insert<T: Send + Sync + 'static>(mut self, value: T) -> Self {
        Storage::insert_value(&mut self.builder.storage, value);
        self
    }

    /// Builds a bot by connecting to Discord and preparing routers.
    pub async fn build(self) -> Result<Bot> {
        self.builder.build().await
    }

    /// Returns the underlying [`BotBuilder`] without building.
    pub fn finish(self) -> BotBuilder {
        self.builder
    }
}
