/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use crate::core::interaction::view::option::view::HasCustomId;
use twilight_model::application::interaction::modal::ModalInteractionComponent;
use twilight_model::id::Id;
use twilight_model::id::marker::{
    AttachmentMarker, ChannelMarker, GenericMarker, RoleMarker, UserMarker,
};

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
            .find_map(|component| match component {
                component if component.custom_id() == Some(custom_id) => Some(component),

                ModalInteractionComponent::ActionRow(row) => row
                    .components
                    .iter()
                    .find(|component| component.custom_id() == Some(custom_id)),

                _ => None,
            })
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
