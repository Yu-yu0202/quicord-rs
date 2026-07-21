/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use crate::Bot;
use crate::command::context::{MessageContextCommandMetadata, UserContextCommandMetadata};
use crate::command::message_component::{ButtonMetadata, SelectMenuMetadata};
use crate::command::modal::ModalMetadata;
use crate::command::slash::SlashCommandMetadata;
use crate::core::event::EventHandlerMetadata;
use twilight_model::application::command::CommandType;
use twilight_model::application::interaction::InteractionData;
use twilight_model::application::interaction::message_component::MessageComponentInteractionData;
use twilight_model::channel::message::component::ComponentType;
use twilight_model::gateway::event::Event;

/// Routed handler resolved from an incoming event.
pub(crate) enum RoutedHandler {
    /// A gateway event handler.
    Event(&'static EventHandlerMetadata),
    /// A slash command handler.
    Slash(&'static SlashCommandMetadata),
    /// A user context command handler.
    UserContext(&'static UserContextCommandMetadata),
    /// A message context command handler.
    MessageContext(&'static MessageContextCommandMetadata),
    /// A button interaction handler.
    Button(&'static ButtonMetadata),
    /// A select menu interaction handler.
    SelectMenu(&'static SelectMenuMetadata),
    /// A modal interaction handler.
    Modal(&'static ModalMetadata),
}

impl Bot {
    /// Resolves message component interactions to their corresponding handlers based on the custom ID.
    fn route_component(
        &self,
        component: &MessageComponentInteractionData,
    ) -> Option<RoutedHandler> {
        match component.component_type {
            ComponentType::Button => self
                .button_router
                .get(component.custom_id.as_str())
                .map(RoutedHandler::Button),

            ComponentType::MentionableSelectMenu
            | ComponentType::RoleSelectMenu
            | ComponentType::UserSelectMenu
            | ComponentType::ChannelSelectMenu => self
                .select_menu_router
                .get(component.custom_id.as_str())
                .map(RoutedHandler::SelectMenu),

            _ => None,
        }
    }

    /// Resolves an incoming event into a registered handler, if any.
    pub(crate) fn route_event(&self, event: &Event) -> Option<RoutedHandler> {
        match event {
            Event::InteractionCreate(interaction) => match interaction.data.as_ref()? {
                InteractionData::ApplicationCommand(cmd) => {
                    self.route_application_command(cmd.kind, &cmd.name)
                }
                InteractionData::MessageComponent(component) => self.route_component(component),
                InteractionData::ModalSubmit(modal) => self
                    .modal_router
                    .get(modal.custom_id.as_str())
                    .map(RoutedHandler::Modal),
                _ => None,
            },
            _ => self.route_gateway_event(event),
        }
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
