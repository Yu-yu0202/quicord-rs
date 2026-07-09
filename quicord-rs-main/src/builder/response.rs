/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
use crate::builder::button::ButtonBuilder;
use crate::builder::text_input::TextInputBuilder;
use crate::core::interaction::r#trait::IntoResponse;
use twilight_model::application::command::CommandOptionChoice;
use twilight_model::channel::message::component::ActionRow;
use twilight_model::channel::message::{AllowedMentions, Component, Embed, MessageFlags};
use twilight_model::http::attachment::Attachment;
use twilight_model::http::interaction::InteractionResponseData;
use twilight_model::poll::Poll;

pub struct ResponseBuilder(InteractionResponseData);

impl ResponseBuilder {
    pub fn new() -> Self {
        Self(InteractionResponseData {
            allowed_mentions: None,
            attachments: None,
            choices: None,
            components: None,
            content: None,
            custom_id: None,
            embeds: None,
            flags: None,
            title: None,
            tts: None,
            poll: None,
        })
    }

    pub fn build(self) -> InteractionResponseData {
        self.0
    }

    pub fn allowed_mentions(mut self, allowed_mentions: impl Into<AllowedMentions>) -> Self {
        self.0.allowed_mentions = Some(allowed_mentions.into());
        self
    }

    pub fn attachments(mut self, attachments: impl Into<Vec<Attachment>>) -> Self {
        self.0.attachments = Some(attachments.into());
        self
    }

    pub fn attachment(mut self, attachment: impl Into<Attachment>) -> Self {
        let attachments = self.0.attachments.get_or_insert_with(Vec::new);
        attachments.push(attachment.into());
        self
    }

    pub fn choices(
        mut self,
        choices: impl IntoIterator<Item = impl Into<CommandOptionChoice>>,
    ) -> Self {
        self.0.choices = Some(choices.into_iter().map(|x| x.into()).collect());
        self
    }

    pub fn choice(mut self, choice: impl Into<CommandOptionChoice>) -> Self {
        let choices = self.0.choices.get_or_insert_with(Vec::new);
        choices.push(choice.into());
        self
    }

    pub fn components(
        mut self,
        components: impl IntoIterator<Item = impl Into<Component>>,
    ) -> Self {
        self.0.components = Some(components.into_iter().map(|x| x.into()).collect());
        self
    }

    pub fn component(mut self, component: impl Into<Component>) -> Self {
        let components = self.0.components.get_or_insert_with(Vec::new);
        components.push(component.into());
        self
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.0.content = Some(content.into());
        self
    }

    pub fn custom_id(mut self, custom_id: impl Into<String>) -> Self {
        self.0.custom_id = Some(custom_id.into());
        self
    }

    pub fn embeds(mut self, embeds: impl IntoIterator<Item = impl Into<Embed>>) -> Self {
        self.0.embeds = Some(embeds.into_iter().map(|x| x.into()).collect());
        self
    }

    pub fn embed(mut self, embed: impl Into<Embed>) -> Self {
        let embeds = self.0.embeds.get_or_insert_with(Vec::new);
        embeds.push(embed.into());
        self
    }

    pub fn flags(mut self, flags: impl Into<MessageFlags>) -> Self {
        self.0.flags = Some(flags.into());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.0.title = Some(title.into());
        self
    }

    pub fn tts(mut self, tts: bool) -> Self {
        self.0.tts = Some(tts);
        self
    }

    pub fn poll(mut self, poll: Poll) -> Self {
        self.0.poll = Some(poll);
        self
    }

    pub fn buttons(
        mut self,
        component: impl IntoIterator<Item = impl Into<ButtonBuilder>>,
    ) -> Self {
        for c in component {
            self.push_action_row_component(c.into());
        }
        self
    }

    pub fn button(mut self, component: impl Into<ButtonBuilder>) -> Self {
        self.push_action_row_component(component.into());
        self
    }

    pub fn text_inputs(
        mut self,
        component: impl IntoIterator<Item = impl Into<TextInputBuilder>>,
    ) -> Self {
        for c in component {
            self.push_action_row_component(c.into());
        }
        self
    }

    pub fn text_input(mut self, component: impl Into<TextInputBuilder>) -> Self {
        self.push_action_row_component(component.into());
        self
    }

    fn push_action_row_component(&mut self, component: impl Into<ActionRow>) {
        self.0
            .components
            .get_or_insert_with(Vec::new)
            .push(component.into().into());
    }
}

impl From<ResponseBuilder> for InteractionResponseData {
    fn from(builder: ResponseBuilder) -> Self {
        builder.build()
    }
}

impl IntoResponse for ResponseBuilder {
    fn into_response(self) -> InteractionResponseData {
        self.build()
    }
}
