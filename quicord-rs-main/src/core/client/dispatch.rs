/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
use crate::core::client::router::RoutedHandler;
use crate::core::client::Client;
use crate::{Bot, EventContext, InteractionContext};
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::sync::Mutex;
use tracing::{info, warn};
use twilight_model::gateway::event::Event;

lazy_static! {
    static ref EXECUTED_ONCE_EVENT_HANDLERS: Mutex<HashSet<usize>> = Mutex::new(HashSet::new());
}

impl Bot {
    /// Dispatches a routed event to the associated handler.
    pub(crate) async fn handle_routed_event(client: Client, handler: RoutedHandler, event: Event) {
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
                let context = InteractionContext::new(client, event);
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
                let context = InteractionContext::new(client, event);
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
                let context = InteractionContext::new(client, event);
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
                let context = InteractionContext::new(client, event);
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
                let context = InteractionContext::new(client, event);
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

fn log_executor_info(context: &InteractionContext) -> () {
    if let Some(user) = context.author() {
        let display_name = user.global_name.as_deref().unwrap_or("<unknown>");
        let user_name = user.name.as_str();
        let user_id = user.id.to_string();
        info!("Executed by: {display_name} (name: {user_name}, ID: {user_id})");
    }
}
