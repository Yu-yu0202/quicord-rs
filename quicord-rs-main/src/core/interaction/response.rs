/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use crate::InteractionContext;
use crate::core::interaction::r#trait::IntoResponse;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::{
    Id,
    marker::{ApplicationMarker, InteractionMarker},
};

impl InteractionContext {
    fn interaction_target(&self) -> Option<InteractionTarget<'_>> {
        self.interaction().map(|interaction| InteractionTarget {
            application_id: interaction.application_id,
            id: interaction.id,
            token: &interaction.token,
        })
    }

    /// Sends a channel message response for the interaction.
    pub async fn reply(&self, response: impl IntoResponse) -> anyhow::Result<()> {
        self.create_response(
            InteractionResponseType::ChannelMessageWithSource,
            Some(response.into_response()),
        )
        .await
    }

    /// Defers the initial response and optionally marks it ephemeral.
    pub async fn defer_reply(&self, ephemeral: bool) -> anyhow::Result<()> {
        self.create_response(
            InteractionResponseType::DeferredChannelMessageWithSource,
            ephemeral.then(ephemeral_response_data),
        )
        .await
    }

    /// Edits the original response message.
    pub async fn edit_reply(&self, response: impl IntoResponse) -> anyhow::Result<()> {
        if let Some(target) = self.interaction_target() {
            let data = response.into_response();
            let json = serde_json::to_vec(&data)?;

            self.client
                .http
                .interaction(target.application_id)
                .update_response(target.token)
                .payload_json(&json)
                .await?;
        }

        Ok(())
    }

    /// Sends a modal response for the interaction.
    pub async fn show_modal(
        &self,
        modal: impl Into<InteractionResponseData>,
    ) -> anyhow::Result<()> {
        self.create_response(InteractionResponseType::Modal, Some(modal.into()))
            .await
    }

    /// Updates the original response message.
    pub async fn update(&self, response: impl IntoResponse) -> anyhow::Result<()> {
        self.create_response(
            InteractionResponseType::UpdateMessage,
            Some(response.into_response()),
        )
        .await
    }

    /// Defers the update of the original response message and optionally marks it ephemeral.
    pub async fn defer_update(&self, ephemeral: bool) -> anyhow::Result<()> {
        self.create_response(
            InteractionResponseType::DeferredUpdateMessage,
            ephemeral.then(ephemeral_response_data),
        )
        .await
    }

    /// Sends a raw interaction response to Discord.
    pub async fn create_response(
        &self,
        kind: InteractionResponseType,
        data: Option<InteractionResponseData>,
    ) -> anyhow::Result<()> {
        if let Some(target) = self.interaction_target() {
            let payload = InteractionResponse { kind, data };

            self.client
                .http
                .interaction(target.application_id)
                .create_response(target.id, target.token, &payload)
                .await?;
        }

        Ok(())
    }
}

struct InteractionTarget<'a> {
    application_id: Id<ApplicationMarker>,
    id: Id<InteractionMarker>,
    token: &'a str,
}

/// Builds the payload used for ephemeral interaction responses.
fn ephemeral_response_data() -> InteractionResponseData {
    InteractionResponseData {
        flags: Some(MessageFlags::EPHEMERAL),
        ..Default::default()
    }
}
