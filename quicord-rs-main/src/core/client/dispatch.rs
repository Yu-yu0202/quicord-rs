/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use crate::core::client::Client;
use crate::core::client::router::RoutedHandler;
use crate::core::event::EventRegistry;
use crate::core::storage::Storage;
use crate::{Bot, EventContext, InteractionContext};
use tracing::{info, warn};
use twilight_model::gateway::event::Event;

impl Bot {
    /// Dispatches a routed event to the associated handler.
    pub(crate) async fn handle_routed_event(
        client: Client,
        storage: Storage,
        event_registry: EventRegistry,
        handler: RoutedHandler,
        event: Event,
    ) {
        match handler {
            RoutedHandler::Event(event_route) => {
                for hook in event_route.hooks {
                    if !hook.should_execute() {
                        continue;
                    }

                    info!("Handling event: {}", event_route.event_type);
                    let context = EventContext::with_registry(
                        client.clone(),
                        storage.clone(),
                        event.clone(),
                        event_registry.clone(),
                    );
                    if let Err(e) = hook.invoke(context).await {
                        warn!("Error handling event {}: {:?}", event_route.event_type, e);
                    } else {
                        info!("Successfully handled event: {}", event_route.event_type);
                    }
                }
            }
            RoutedHandler::Slash(command_meta) => {
                info!("Handling slash command: /{}", command_meta.name);
                let context =
                    InteractionContext::new(client, storage.clone(), event, event_registry);
                log_executor_info(&context);
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
                let context =
                    InteractionContext::new(client, storage.clone(), event, event_registry);
                log_executor_info(&context);
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
                let context =
                    InteractionContext::new(client, storage.clone(), event, event_registry);
                log_executor_info(&context);
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
            RoutedHandler::Modal(modal_meta) => {
                info!("Handling modal submission: {}", modal_meta.custom_id);
                let context =
                    InteractionContext::new(client, storage.clone(), event, event_registry);
                log_executor_info(&context);
                if let Err(e) = (modal_meta.run)(context).await {
                    warn!(
                        "Error handling modal submission {}: {:?}",
                        modal_meta.custom_id, e
                    );
                } else {
                    info!(
                        "Successfully handled modal submission: {}",
                        modal_meta.custom_id
                    );
                }
            }
            RoutedHandler::Button(button_meta) => {
                info!("Handling button interaction: {}", button_meta.custom_id);
                let context =
                    InteractionContext::new(client, storage.clone(), event, event_registry);
                log_executor_info(&context);
                if let Err(e) = (button_meta.run)(context).await {
                    warn!(
                        "Error handling button interaction {}: {:?}",
                        button_meta.custom_id, e
                    );
                } else {
                    info!(
                        "Successfully handled button interaction: {}",
                        button_meta.custom_id
                    );
                }
            }
            RoutedHandler::SelectMenu(select_menu_meta) => {
                info!(
                    "Handling select menu interaction: {}",
                    select_menu_meta.custom_id
                );
                let context = InteractionContext::new(client, storage, event, event_registry);
                log_executor_info(&context);
                if let Err(e) = (select_menu_meta.run)(context).await {
                    warn!(
                        "Error handling select menu interaction {}: {:?}",
                        select_menu_meta.custom_id, e
                    );
                } else {
                    info!(
                        "Successfully handled select menu interaction: {}",
                        select_menu_meta.custom_id
                    );
                }
            }
        }
    }
}

fn log_executor_info(context: &InteractionContext) {
    if let Some(user) = context.author() {
        let display_name = user.global_name.as_deref().unwrap_or("<unknown>");
        let user_name = user.name.as_str();
        let user_id = user.id.to_string();
        info!("Executed by: {display_name} (name: {user_name}, ID: {user_id})");
    }
}
