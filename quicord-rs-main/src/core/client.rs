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
            MESSAGE_CONTEXT_COMMANDS, MessageContextCommandMetadata, USER_CONTEXT_COMMANDS,
            UserContextCommandMetadata,
        },
        slash::{SLASH_COMMANDS, SlashCommandMetadata},
    },
    core::event::{EVENT_HANDLERS, EventHookId, EventRegistry, INTERNAL_EVENT_HANDLERS},
    core::storage::Storage,
    util::static_router::StaticRouter,
};
use anyhow::Result;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::future::Future;
use std::num::{NonZeroU32, NonZeroUsize};
use std::sync::Arc;
use std::time::Duration;
use twilight_gateway::{ConfigBuilder, Shard};
use twilight_http::Client as HttpClient;
use twilight_model::id::{Id, marker::ApplicationMarker};

pub use twilight_gateway::Intents;
pub use twilight_model::gateway::ShardId;

mod dispatch;
mod register;
mod router;
mod runtime;
mod tls;

pub use runtime::BotTask;

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
    intents: Intents,
    shard_id: ShardId,
    max_concurrent_handlers: NonZeroUsize,
    task_shutdown_policy: TaskShutdownPolicy,
    gateway_retry_policy: GatewayRetryPolicy,
}

/// Fluent builder for registering shared [`Storage`] values before building a [`Bot`].
pub struct BotBuilderStorage {
    builder: BotBuilder,
}

/// Static registries prepared once when a bot is built.
struct BotRouters {
    event_registry: EventRegistry,
    slash_router: StaticRouter<&'static str, SlashCommandMetadata>,
    user_context_router: StaticRouter<&'static str, UserContextCommandMetadata>,
    message_context_router: StaticRouter<&'static str, MessageContextCommandMetadata>,
    modal_router: StaticRouter<&'static str, ModalMetadata>,
    button_router: StaticRouter<&'static str, ButtonMetadata>,
    select_menu_router: StaticRouter<&'static str, SelectMenuMetadata>,
}

impl BotRouters {
    fn build() -> Result<Self> {
        let event_registry =
            EventRegistry::from_static(EVENT_HANDLERS.iter().chain(INTERNAL_EVENT_HANDLERS.iter()));
        let slash_router = StaticRouter::try_new(SLASH_COMMANDS.iter(), |metadata| metadata.name)
            .map_err(|key| anyhow::anyhow!("duplicate slash command name: {key}"))?;
        let user_context_router =
            StaticRouter::try_new(USER_CONTEXT_COMMANDS.iter(), |metadata| metadata.name)
                .map_err(|key| anyhow::anyhow!("duplicate user context command name: {key}"))?;
        let message_context_router =
            StaticRouter::try_new(MESSAGE_CONTEXT_COMMANDS.iter(), |metadata| metadata.name)
                .map_err(|key| anyhow::anyhow!("duplicate message context command name: {key}"))?;
        let modal_router =
            StaticRouter::try_new(crate::command::modal::MODALS.iter(), |metadata| {
                metadata.custom_id
            })
            .map_err(|key| anyhow::anyhow!("duplicate modal custom ID: {key}"))?;
        let button_router = StaticRouter::try_new(
            crate::command::message_component::BUTTONS.iter(),
            |metadata| metadata.custom_id,
        )
        .map_err(|key| anyhow::anyhow!("duplicate button custom ID: {key}"))?;
        let select_menu_router = StaticRouter::try_new(
            crate::command::message_component::SELECT_MENUS.iter(),
            |metadata| metadata.custom_id,
        )
        .map_err(|key| anyhow::anyhow!("duplicate select menu custom ID: {key}"))?;

        Ok(Self {
            event_registry,
            slash_router,
            user_context_router,
            message_context_router,
            modal_router,
            button_router,
            select_menu_router,
        })
    }
}

/// How the runtime treats in-flight handlers after shutdown starts.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TaskShutdownPolicy {
    /// Wait for all handlers that were already started to complete.
    #[default]
    Drain,
    /// Cancel all in-flight handlers before returning from the event loop.
    Abort,
}

/// Policy for retrying failed gateway reconnections.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GatewayRetryPolicy {
    initial_delay: Duration,
    max_delay: Duration,
    max_attempts: NonZeroU32,
}

impl GatewayRetryPolicy {
    /// Creates a retry policy with capped exponential backoff.
    ///
    /// `initial_delay` is used after the first failed reconnect. Each later
    /// failure doubles the delay until `max_delay` is reached. The runtime
    /// stops after `max_attempts` consecutive failed reconnects.
    pub const fn new(
        initial_delay: Duration,
        max_delay: Duration,
        max_attempts: NonZeroU32,
    ) -> Self {
        Self {
            initial_delay,
            max_delay,
            max_attempts,
        }
    }

    pub(crate) fn delay_after_attempt(self, attempt: u32) -> Option<Duration> {
        if attempt > self.max_attempts.get() {
            return None;
        }

        let multiplier = 1_u32 << attempt.saturating_sub(1).min(31);
        Some(
            self.initial_delay
                .saturating_mul(multiplier)
                .min(self.max_delay),
        )
    }

    pub(crate) const fn max_attempts(self) -> NonZeroU32 {
        self.max_attempts
    }
}

impl Default for GatewayRetryPolicy {
    fn default() -> Self {
        Self::new(
            Duration::from_secs(1),
            Duration::from_secs(30),
            NonZeroU32::new(5).expect("5 is non-zero"),
        )
    }
}

/// Bot runtime state and routing tables.
pub struct Bot {
    /// Shared client access for handlers.
    pub client: Client,
    /// The application ID resolved from Discord.
    pub application_id: Id<ApplicationMarker>,
    pub(crate) shard: Shard,
    pub(crate) storage: Storage,

    event_registry: EventRegistry,
    slash_router: StaticRouter<&'static str, SlashCommandMetadata>,
    user_context_router: StaticRouter<&'static str, UserContextCommandMetadata>,
    message_context_router: StaticRouter<&'static str, MessageContextCommandMetadata>,
    modal_router: StaticRouter<&'static str, ModalMetadata>,
    button_router: StaticRouter<&'static str, ButtonMetadata>,
    select_menu_router: StaticRouter<&'static str, SelectMenuMetadata>,
    max_concurrent_handlers: NonZeroUsize,
    task_shutdown_policy: TaskShutdownPolicy,
    gateway_retry_policy: GatewayRetryPolicy,
}

impl Client {
    /// Creates a new client wrapper from an HTTP client.
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

impl Bot {
    /// Registers an internal event hook for this bot.
    #[allow(dead_code)]
    pub(crate) fn register_event<F, Fut>(
        &self,
        event_type: impl AsRef<str>,
        handler: F,
    ) -> EventHookId
    where
        F: Fn(crate::EventContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        self.event_registry.register(event_type, handler, false)
    }

    /// Registers an internal event hook that runs at most once for this bot.
    #[allow(dead_code)]
    pub(crate) fn register_event_once<F, Fut>(
        &self,
        event_type: impl AsRef<str>,
        handler: F,
    ) -> EventHookId
    where
        F: Fn(crate::EventContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        self.event_registry.register_once(event_type, handler)
    }

    /// Removes an internal event hook from future dispatches.
    #[allow(dead_code)]
    pub(crate) fn unregister_event(&self, id: EventHookId) -> bool {
        self.event_registry.unregister(id)
    }
}

impl BotBuilder {
    /// Creates a new builder using the provided bot token.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            storage: HashMap::new(),
            intents: Intents::all(),
            shard_id: ShardId::ONE,
            max_concurrent_handlers: NonZeroUsize::MIN,
            task_shutdown_policy: TaskShutdownPolicy::Drain,
            gateway_retry_policy: GatewayRetryPolicy::default(),
        }
    }

    /// Sets the gateway intents requested by this shard.
    pub fn intents(mut self, intents: Intents) -> Self {
        self.intents = intents;
        self
    }

    /// Sets this runtime's position in the bot's shard topology.
    pub fn shard_id(mut self, shard_id: ShardId) -> Self {
        self.shard_id = shard_id;
        self
    }

    /// Sets the maximum number of handlers that may run at once.
    ///
    /// A limit of one retains sequential dispatch semantics.
    pub fn max_concurrent_handlers(mut self, maximum: NonZeroUsize) -> Self {
        self.max_concurrent_handlers = maximum;
        self
    }

    /// Sets how in-flight handlers are treated during shutdown.
    pub fn task_shutdown_policy(mut self, policy: TaskShutdownPolicy) -> Self {
        self.task_shutdown_policy = policy;
        self
    }

    /// Sets how failed gateway reconnections are retried.
    pub fn gateway_retry_policy(mut self, policy: GatewayRetryPolicy) -> Self {
        self.gateway_retry_policy = policy;
        self
    }

    /// Returns a fluent builder for registering shared storage values.
    pub fn storage(self) -> BotBuilderStorage {
        BotBuilderStorage { builder: self }
    }

    /// Returns this builder without discarding its configured state.
    ///
    /// This method is retained for source compatibility. New code can pass the
    /// builder directly to [`Self::build`].
    pub fn share(self) -> BotBuilder {
        self
    }

    /// Builds a bot by connecting to Discord and preparing routers.
    pub async fn build(self) -> Result<Bot> {
        tls::init_rustls();

        let BotBuilder {
            token,
            storage,
            intents,
            shard_id,
            max_concurrent_handlers,
            task_shutdown_policy,
            gateway_retry_policy,
        } = self;
        let routers = BotRouters::build()?;

        let http = Arc::new(HttpClient::new(token.clone()));

        let config = ConfigBuilder::new(token, intents).build();
        let shard = Shard::with_config(shard_id, config);

        let application_id = http.current_user_application().await?.model().await?.id;

        Ok(Bot {
            client: Client::new(http),
            shard,
            application_id,
            storage: Storage::new(storage),
            event_registry: routers.event_registry,
            slash_router: routers.slash_router,
            user_context_router: routers.user_context_router,
            message_context_router: routers.message_context_router,
            modal_router: routers.modal_router,
            button_router: routers.button_router,
            select_menu_router: routers.select_menu_router,
            max_concurrent_handlers,
            task_shutdown_policy,
            gateway_retry_policy,
        })
    }
}

impl Bot {
    /// Constructs a bot from the provided builder.
    pub async fn new(config: BotBuilder) -> Result<Self> {
        config.build().await
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

#[cfg(test)]
mod tests {
    use super::BotBuilder;
    use std::any::TypeId;

    #[test]
    fn share_preserves_registered_storage() {
        let builder = BotBuilder::new("token")
            .storage()
            .insert(42_u32)
            .finish()
            .share();

        assert!(builder.storage.contains_key(&TypeId::of::<u32>()));
    }
}
