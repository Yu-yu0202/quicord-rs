/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
use twilight_model::channel::message::component::{ActionRow, TextInput};
use twilight_model::channel::message::Component;

pub struct TextInputBuilder(TextInput);

impl TextInputBuilder {
    fn new(
        custom_id: impl Into<String>,
        style: twilight_model::channel::message::component::TextInputStyle,
    ) -> Self {
        Self(TextInput {
            custom_id: custom_id.into(),
            id: None,
            #[allow(deprecated)]
            label: None,
            max_length: None,
            min_length: None,
            placeholder: None,
            required: None,
            style,
            value: None,
        })
    }

    pub fn short(custom_id: impl Into<String>) -> Self {
        Self::new(
            custom_id,
            twilight_model::channel::message::component::TextInputStyle::Short,
        )
    }

    pub fn paragraph(custom_id: impl Into<String>) -> Self {
        Self::new(
            custom_id,
            twilight_model::channel::message::component::TextInputStyle::Paragraph,
        )
    }

    pub fn id(mut self, id: impl Into<i32>) -> Self {
        self.0.id = Some(id.into());
        self
    }

    #[allow(deprecated)]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.0.label = Some(label.into());
        self
    }

    pub fn max_length(mut self, max_length: impl Into<u16>) -> Self {
        self.0.max_length = Some(max_length.into());
        self
    }

    pub fn min_length(mut self, min_length: impl Into<u16>) -> Self {
        self.0.min_length = Some(min_length.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.0.placeholder = Some(placeholder.into());
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.0.required = Some(required);
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.0.value = Some(value.into());
        self
    }

    pub fn get(&self) -> &TextInput {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut TextInput {
        &mut self.0
    }

    pub fn into_row_component(self) -> Component {
        Component::ActionRow(self.into())
    }

    pub fn build(self) -> TextInput {
        self.0
    }
}

impl From<TextInputBuilder> for TextInput {
    fn from(builder: TextInputBuilder) -> Self {
        builder.build()
    }
}

impl From<TextInputBuilder> for Component {
    fn from(builder: TextInputBuilder) -> Self {
        Component::TextInput(builder.build())
    }
}

impl From<TextInputBuilder> for ActionRow {
    fn from(builder: TextInputBuilder) -> Self {
        ActionRow {
            id: None,
            components: vec![Component::TextInput(builder.build())],
        }
    }
}
