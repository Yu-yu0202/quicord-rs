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

impl InteractionContext {
    /// Sends a channel message response for the interaction.
    pub async fn reply(&self, response: impl IntoResponse) -> anyhow::Result<()> {
        if self.interaction().is_some() {
            let data = response.into_response();

            self.create_response(
                InteractionResponseType::ChannelMessageWithSource,
                Some(data),
            )
            .await?;
        }

        Ok(())
    }

    /// Defers the initial response and optionally marks it ephemeral.
    pub async fn defer_reply(&self, ephemeral: bool) -> anyhow::Result<()> {
        if self.interaction().is_some() {
            self.create_response(
                InteractionResponseType::DeferredChannelMessageWithSource,
                ephemeral.then(ephemeral_response_data),
            )
            .await?;
        }

        Ok(())
    }

    /// Edits the original response message.
    pub async fn edit_reply(&self, response: impl IntoResponse) -> anyhow::Result<()> {
        if let Some(interaction) = self.interaction() {
            let data = response.into_response();
            let json = serde_json::to_vec(&data)?;

            self.client
                .http
                .interaction(interaction.application_id)
                .update_response(&interaction.token)
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
        if self.interaction().is_some() {
            let data = modal.into();

            self.create_response(InteractionResponseType::Modal, Some(data))
                .await?;
        }

        Ok(())
    }

    /// Updates the original response message.
    pub async fn update(&self, response: impl IntoResponse) -> anyhow::Result<()> {
        if self.interaction().is_some() {
            let data = response.into_response();

            self.create_response(InteractionResponseType::UpdateMessage, Some(data))
                .await?;
        }

        Ok(())
    }

    /// Defers the update of the original response message and optionally marks it ephemeral.
    pub async fn defer_update(&self, ephemeral: bool) -> anyhow::Result<()> {
        if self.interaction().is_some() {
            self.create_response(
                InteractionResponseType::DeferredUpdateMessage,
                ephemeral.then(ephemeral_response_data),
            )
            .await?;
        }

        Ok(())
    }

    /// Sends a raw interaction response to Discord.
    pub async fn create_response(
        &self,
        kind: InteractionResponseType,
        data: Option<InteractionResponseData>,
    ) -> anyhow::Result<()> {
        if let Some(interaction) = self.interaction() {
            let payload = InteractionResponse { kind, data };

            self.client
                .http
                .interaction(interaction.application_id)
                .create_response(interaction.id, &interaction.token, &payload)
                .await?;
        }

        Ok(())
    }
}

/// Builds the payload used for ephemeral interaction responses.
fn ephemeral_response_data() -> InteractionResponseData {
    InteractionResponseData {
        flags: Some(MessageFlags::EPHEMERAL),
        ..Default::default()
    }
}
