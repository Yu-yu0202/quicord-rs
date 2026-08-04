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
use crate::core::event::{EventRegistry, RegisteredEventHook};
use crate::core::storage::Storage;
use crate::{Bot, EventContext, InteractionContext};
use std::sync::Arc;
use tracing::{info, warn};
use twilight_model::gateway::event::Event;

/// Runtime state cloned into one independently executable routed task.
#[derive(Clone)]
pub(crate) struct DispatchState {
    client: Client,
    storage: Storage,
    event_registry: EventRegistry,
}

/// An owned execution plan for one routed gateway event.
pub(crate) enum RoutedEventTask {
    Event {
        state: DispatchState,
        event_type: String,
        hooks: Vec<Arc<RegisteredEventHook>>,
        event: Event,
    },
    Interaction {
        state: DispatchState,
        kind: &'static str,
        identifier: &'static str,
        handler: crate::command::CommandHandler,
        event: Event,
    },
}

impl Bot {
    /// Produces an executable task from a routed event without running it.
    pub(crate) fn prepare_routed_event(
        &self,
        handler: RoutedHandler,
        event: Event,
    ) -> Option<RoutedEventTask> {
        let state = DispatchState {
            client: self.client.clone(),
            storage: self.storage.clone(),
            event_registry: self.event_registry.clone(),
        };

        RoutedEventTask::prepare(state, handler, event)
    }
}

impl RoutedEventTask {
    /// Converts routing metadata into an owned execution plan.
    fn prepare(state: DispatchState, handler: RoutedHandler, event: Event) -> Option<Self> {
        match handler {
            RoutedHandler::Event(event_route) => {
                let hooks = event_route
                    .hooks
                    .into_iter()
                    .filter(|hook| hook.try_claim_execution())
                    .collect::<Vec<_>>();

                (!hooks.is_empty()).then_some(Self::Event {
                    state,
                    event_type: event_route.event_type,
                    hooks,
                    event,
                })
            }
            RoutedHandler::Slash(metadata) => Some(Self::interaction(
                state,
                "slash command",
                metadata.name,
                metadata.run,
                event,
            )),
            RoutedHandler::UserContext(metadata) => Some(Self::interaction(
                state,
                "user context command",
                metadata.name,
                metadata.run,
                event,
            )),
            RoutedHandler::MessageContext(metadata) => Some(Self::interaction(
                state,
                "message context command",
                metadata.name,
                metadata.run,
                event,
            )),
            RoutedHandler::Modal(metadata) => Some(Self::interaction(
                state,
                "modal submission",
                metadata.custom_id,
                metadata.run,
                event,
            )),
            RoutedHandler::Button(metadata) => Some(Self::interaction(
                state,
                "button interaction",
                metadata.custom_id,
                metadata.run,
                event,
            )),
            RoutedHandler::SelectMenu(metadata) => Some(Self::interaction(
                state,
                "select menu interaction",
                metadata.custom_id,
                metadata.run,
                event,
            )),
        }
    }

    fn interaction(
        state: DispatchState,
        kind: &'static str,
        identifier: &'static str,
        handler: crate::command::CommandHandler,
        event: Event,
    ) -> Self {
        Self::Interaction {
            state,
            kind,
            identifier,
            handler,
            event,
        }
    }

    /// Executes the task and records handler-local failures.
    pub(crate) async fn run(self) {
        match self {
            Self::Event {
                state,
                event_type,
                hooks,
                event,
            } => {
                for hook in hooks {
                    info!(event_type = %event_type, "Handling event");
                    let context = EventContext::with_registry(
                        state.client.clone(),
                        state.storage.clone(),
                        event.clone(),
                        state.event_registry.clone(),
                    );
                    if let Err(e) = hook.invoke(context).await {
                        warn!(event_type = %event_type, error = ?e, "Error handling event");
                    } else {
                        info!(event_type = %event_type, "Successfully handled event");
                    }
                }
            }
            Self::Interaction {
                state,
                kind,
                identifier,
                handler,
                event,
            } => {
                info!(kind, identifier, "Handling interaction");
                let context = InteractionContext::new(
                    state.client,
                    state.storage,
                    event,
                    state.event_registry,
                );
                log_executor_info(&context);
                if let Err(e) = handler(context).await {
                    warn!(kind, identifier, error = ?e, "Error handling interaction");
                } else {
                    info!(kind, identifier, "Successfully handled interaction");
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

#[cfg(test)]
mod tests {
    use super::RoutedEventTask;

    #[test]
    fn routed_event_tasks_are_send() {
        fn assert_send<T: Send>() {}

        assert_send::<RoutedEventTask>();
    }
}
