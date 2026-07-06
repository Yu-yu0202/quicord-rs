/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
use twilight_model::channel::message::component::{ActionRow, Button};
use twilight_model::channel::message::Component;

pub use twilight_model::channel::message::component::ButtonStyle;
use twilight_model::id::marker::SkuMarker;
use twilight_model::id::Id;

pub struct ButtonBuilder(Button);

impl ButtonBuilder {
    pub fn new(style: ButtonStyle) -> Self {
        Self(Button {
            id: None,
            custom_id: None,
            disabled: false,
            emoji: None,
            label: None,
            style,
            url: None,
            sku_id: None,
        })
    }

    pub fn custom_id(mut self, custom_id: impl Into<String>) -> Self {
        self.0.custom_id = Some(custom_id.into());
        self
    }

    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.0.disabled = disabled;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.0.label = Some(label.into());
        self
    }

    pub fn emoji(
        mut self,
        emoji: impl Into<twilight_model::channel::message::EmojiReactionType>,
    ) -> Self {
        self.0.emoji = Some(emoji.into());
        self
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.0.url = Some(url.into());
        self
    }

    pub fn sku_id(mut self, sku_id: Id<SkuMarker>) -> Self {
        self.0.sku_id = Some(sku_id);
        self
    }

    pub fn get(&self) -> &Button {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut Button {
        &mut self.0
    }

    pub fn into_row_component(self) -> Component {
        Component::ActionRow(self.into())
    }

    pub fn build(self) -> Button {
        self.0
    }
}

impl From<ButtonBuilder> for Button {
    fn from(builder: ButtonBuilder) -> Self {
        builder.build()
    }
}

impl From<ButtonBuilder> for Component {
    fn from(value: ButtonBuilder) -> Self {
        Component::Button(value.build())
    }
}

impl From<ButtonBuilder> for ActionRow {
    fn from(value: ButtonBuilder) -> Self {
        ActionRow {
            id: None,
            components: vec![Component::Button(value.build())],
        }
    }
}
