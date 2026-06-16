/*
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
use crate::util::logger::init_logger;
use crate::{
    command::{
        context::{
            MessageContextCommandMetadata, UserContextCommandMetadata, MESSAGE_CONTEXT_COMMANDS,
            USER_CONTEXT_COMMANDS,
        },
        scope::CommandScope,
        slash::{SlashCommandMetadata, SLASH_COMMANDS},
    },
    core::{
        event::{EventContext, EventHandlerMetadata, EVENT_HANDLERS},
        interaction::InteractionContext,
    },
    util::static_router::StaticRouter,
};
use anyhow::Result;
use lazy_static::lazy_static;
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tokio::signal::unix::{signal, SignalKind};
use tracing::{debug, error, info, warn};
use twilight_gateway::{ConfigBuilder, EventTypeFlags, Intents, Shard, StreamExt};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::CloseFrame;
use twilight_model::{
    application::{
        command::{Command, CommandOption, CommandType},
        interaction::InteractionData,
    },
    gateway::{event::Event, ShardId},
    id::{marker::ApplicationMarker, Id},
};
use twilight_util::builder::command::CommandBuilder;

/// Guild ID to command list mapping used while registering commands.
type GuildCommandMap = FxHashMap<&'static str, Vec<Command>>;

/// One-time initializer guard for rustls.
static INIT_RUSTLS: std::sync::Once = std::sync::Once::new();

/// Installs the default rustls crypto provider once per process.
fn init_rustls() -> () {
    INIT_RUSTLS.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

lazy_static! {
    static ref EXECUTED_ONCE_EVENT_HANDLERS: Mutex<HashSet<usize>> = Mutex::new(HashSet::new());
}

/// Routed handler resolved from an incoming event.
pub enum RoutedHandler {
    /// A gateway event handler.
    Event(&'static EventHandlerMetadata),
    /// A slash command handler.
    Slash(&'static SlashCommandMetadata),
    /// A user context command handler.
    UserContext(&'static UserContextCommandMetadata),
    /// A message context command handler.
    MessageContext(&'static MessageContextCommandMetadata),
}

/// Shared HTTP client wrapper used by the bot runtime.
#[derive(Clone)]
pub struct Client {
    /// The underlying Twilight HTTP client.
    pub http: Arc<HttpClient>,
}

/// Builder used to construct a [`Bot`].
pub struct BotBuilder {
    token: String,
}

/// Bot runtime state and routing tables.
pub struct Bot {
    /// Shared client access for handlers.
    pub client: Client,
    /// The application ID resolved from Discord.
    pub application_id: Id<ApplicationMarker>,
    pub(crate) shard: Shard,

    event_router: StaticRouter<&'static str, EventHandlerMetadata>,
    slash_router: StaticRouter<&'static str, SlashCommandMetadata>,
    user_context_router: StaticRouter<&'static str, UserContextCommandMetadata>,
    message_context_router: StaticRouter<&'static str, MessageContextCommandMetadata>,
}

/// Pending command registrations grouped by scope.
struct PendingCommands {
    global: Vec<Command>,
    guild: GuildCommandMap,
}

impl Client {
    /// Creates a new client wrapper from an HTTP client.
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

impl PendingCommands {
    /// Creates an empty command collection.
    fn new() -> Self {
        Self {
            global: Vec::new(),
            guild: GuildCommandMap::with_hasher(FxBuildHasher::default()),
        }
    }

    /// Returns whether no commands are queued for registration.
    fn is_empty(&self) -> bool {
        self.global.is_empty() && self.guild.is_empty()
    }

    /// Adds a command to the appropriate scope buckets.
    fn push(&mut self, scope: CommandScope, command: Command) {
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

impl BotBuilder {
    /// Creates a new builder using the provided bot token.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }

    /// Clones the builder state into a fresh builder.
    pub fn share(self) -> BotBuilder {
        BotBuilder { token: self.token }
    }

    /// Builds a bot by connecting to Discord and preparing routers.
    pub async fn build(self) -> Result<Bot> {
        init_rustls();

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

        Ok(Bot {
            client: Client::new(http),
            shard,
            application_id,
            event_router,
            slash_router,
            user_context_router,
            message_context_router,
        })
    }
}

impl Bot {
    /// Constructs a bot from the provided builder.
    pub async fn new(config: BotBuilder) -> Result<Self> {
        Ok(config.build().await?)
    }

    /// Registers commands and runs the event loop on the current task.
    pub async fn start(mut self) -> Result<()> {
        init_rustls();
        init_logger();

        self.register_commands().await?;

        info!("Starting bot as Application ID {}...", self.application_id);
        self.event_loop().await
    }

    /// Registers commands and spawns the event loop on a background task.
    pub async fn spawn(mut self) -> Result<()> {
        init_rustls();
        init_logger();

        self.register_commands().await?;

        info!(
            "Spawning bot task as Application ID {}...",
            self.application_id
        );

        tokio::spawn(async move {
            if let Err(e) = self.event_loop().await {
                error!("Shard event loop terminated with error: {:?}", e);
            }
        });

        Ok(())
    }

    /// Builds and uploads all registered commands to Discord.
    async fn register_commands(&self) -> Result<()> {
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

    /// Runs the gateway event loop until termination or shutdown.
    async fn event_loop(&mut self) -> Result<()> {
        info!("Starting event loop...");

        #[cfg(unix)]
        let mut sigint = signal(SignalKind::interrupt())?;
        #[cfg(unix)]
        let mut sigterm = signal(SignalKind::terminate())?;

        loop {
            tokio::select! {
                event = self.next_event() => {
                    match event {
                        Some(event) => {
                            debug!("Received event: {:?}", event.kind());

                            if let Some(handler) = self.route_event(&event) {
                                Self::handle_routed_event(self.client.clone(), handler, event).await;
                            }
                        }
                        None => {
                            break;
                        }
                    }
                }

                _ = async {
                    #[cfg(unix)] { sigint.recv().await }
                    #[cfg(not(unix))] { tokio::signal::ctrl_c().await.ok() }
                } => {
                    info!("SIGINT (Ctrl+C) detected. Stopping bot...");
                    self.shard.close(CloseFrame::NORMAL);
                    break;
                }

                _ = async {
                    #[cfg(unix)] { sigterm.recv().await }
                    #[cfg(not(unix))] { std::future::pending::<()>().await }
                } => {
                    info!("SIGTERM detected. Stopping bot...");
                    self.shard.close(CloseFrame::NORMAL);
                    break;
                }
            }
        }

        info!("Stopped bot.");
        Ok(())
    }

    /// Retrieves the next gateway event, retrying transient receive errors.
    async fn next_event(&mut self) -> Option<Event> {
        loop {
            match self.shard.next_event(EventTypeFlags::all()).await {
                Some(Ok(event)) => return Some(event),
                Some(Err(e)) => error!("Error receiving event: {:?}", e),
                None => {
                    error!("Unexpected end of event stream");
                    return None;
                }
            }
        }
    }

    /// Dispatches a routed event to the associated handler.
    async fn handle_routed_event(client: Client, handler: RoutedHandler, event: Event) {
        match handler {
            RoutedHandler::Event(event_meta) => {
                let handler_id = event_meta.handler as usize;

                if event_meta.once {
                    let mut executed = EXECUTED_ONCE_EVENT_HANDLERS
                        .lock()
                        .expect("Failed to access executed handlers list");

                    if executed.contains(&handler_id) {
                        drop(executed);
                        return;
                    }

                    executed.insert(handler_id);
                    drop(executed);
                }

                info!("Handling event: {}", event_meta.event_type);
                let context = EventContext::new(client, event);
                if let Err(e) = (event_meta.handler)(context).await {
                    warn!("Error handling event {}: {:?}", event_meta.event_type, e);
                } else {
                    info!("Successfully handled event: {}", event_meta.event_type);
                }
            }
            RoutedHandler::Slash(command_meta) => {
                info!("Handling slash command: /{}", command_meta.name);
                let context = InteractionContext::new(client, event);
                if let Err(e) = (command_meta.run)(context).await {
                    warn!(
                        "Error handling slash command {}: {:?}",
                        command_meta.name, e
                    );
                } else {
                    info!("Successfully handled slash command: /{}", command_meta.name);
                }
            }
            RoutedHandler::UserContext(command_meta) => {
                info!("Handling user context command: {}", command_meta.name);
                let context = InteractionContext::new(client, event);
                if let Err(e) = (command_meta.run)(context).await {
                    warn!(
                        "Error handling user context command {}: {:?}",
                        command_meta.name, e
                    );
                } else {
                    info!(
                        "Successfully handled user context command: {}",
                        command_meta.name
                    );
                }
            }
            RoutedHandler::MessageContext(command_meta) => {
                info!("Handling message context command: {}", command_meta.name);
                let context = InteractionContext::new(client, event);
                if let Err(e) = (command_meta.run)(context).await {
                    warn!(
                        "Error handling message context command {}: {:?}",
                        command_meta.name, e
                    );
                } else {
                    info!(
                        "Successfully handled message context command: {}",
                        command_meta.name
                    );
                }
            }
        }
    }

    /// Resolves an incoming event into a registered handler, if any.
    pub fn route_event(&self, event: &Event) -> Option<RoutedHandler> {
        if let Event::InteractionCreate(interaction) = event {
            if let Some(InteractionData::ApplicationCommand(ref cmd)) = interaction.data {
                return self.route_application_command(cmd.kind, cmd.name.as_str());
            }
        }

        self.route_gateway_event(event)
    }

    /// Resolves a Discord application command into a command handler.
    fn route_application_command(
        &self,
        command_type: CommandType,
        name: &str,
    ) -> Option<RoutedHandler> {
        match command_type {
            CommandType::ChatInput => self.slash_router.get(name).map(RoutedHandler::Slash),
            CommandType::User => self
                .user_context_router
                .get(name)
                .map(RoutedHandler::UserContext),
            CommandType::Message => self
                .message_context_router
                .get(name)
                .map(RoutedHandler::MessageContext),
            _ => None,
        }
    }

    /// Resolves a gateway event into a gateway event handler.
    fn route_gateway_event(&self, event: &Event) -> Option<RoutedHandler> {
        event
            .kind()
            .name()
            .and_then(|event_type| self.event_router.get(event_type))
            .map(RoutedHandler::Event)
    }
}
