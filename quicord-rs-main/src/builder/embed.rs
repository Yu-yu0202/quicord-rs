/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

pub mod author;
pub mod field;
pub mod footer;
pub mod image;
pub mod provider;
pub mod thumbnail;
pub mod video;

use crate::util::timestamp::now_timestamp;
use twilight_model::channel::message::embed::{
    EmbedAuthor, EmbedField, EmbedFooter, EmbedImage, EmbedProvider, EmbedThumbnail, EmbedVideo,
};
use twilight_model::channel::message::Embed;
use twilight_model::util::Timestamp;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct EmbedBuilder(Embed);

impl EmbedBuilder {
    pub fn new() -> Self {
        EmbedBuilder(Embed {
            author: None,
            color: None,
            description: None,
            fields: Vec::new(),
            footer: None,
            image: None,
            kind: "rich".to_owned(),
            provider: None,
            thumbnail: None,
            timestamp: None,
            title: None,
            url: None,
            video: None,
        })
    }

    pub fn author(mut self, author: impl Into<EmbedAuthor>) -> Self {
        self.0.author = Some(author.into());
        self
    }

    pub fn color(mut self, color: impl Into<u32>) -> Self {
        self.0.color = Some(color.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.0.description = Some(description.into());
        self
    }

    pub fn field(mut self, field: impl Into<EmbedField>) -> Self {
        self.0.fields.push(field.into());
        self
    }

    pub fn fields(mut self, fields: impl IntoIterator<Item = impl Into<EmbedField>>) -> Self {
        self.0.fields.extend(fields.into_iter().map(Into::into));
        self
    }

    pub fn footer(mut self, footer: impl Into<EmbedFooter>) -> Self {
        self.0.footer = Some(footer.into());
        self
    }

    pub fn image(mut self, image: impl Into<EmbedImage>) -> Self {
        self.0.image = Some(image.into());
        self
    }

    pub fn provider(mut self, provider: impl Into<EmbedProvider>) -> Self {
        self.0.provider = Some(provider.into());
        self
    }

    pub fn thumbnail(mut self, thumbnail: impl Into<EmbedThumbnail>) -> Self {
        self.0.thumbnail = Some(thumbnail.into());
        self
    }

    pub fn timestamp(mut self, timestamp: impl Into<Timestamp>) -> Self {
        self.0.timestamp = Some(timestamp.into());
        self
    }

    pub fn timestamp_now(mut self) -> Self {
        self.0.timestamp = Some(now_timestamp());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.0.title = Some(title.into());
        self
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.0.url = Some(url.into());
        self
    }

    pub fn video(mut self, video: impl Into<EmbedVideo>) -> Self {
        self.0.video = Some(video.into());
        self
    }

    pub fn build(self) -> Embed {
        self.0
    }
}

impl Default for EmbedBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<EmbedBuilder> for Embed {
    fn from(builder: EmbedBuilder) -> Self {
        builder.build()
    }
}
