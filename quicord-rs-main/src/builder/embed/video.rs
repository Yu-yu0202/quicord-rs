/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
use twilight_model::channel::message::embed::EmbedVideo;

pub struct EmbedVideoBuilder(EmbedVideo);

impl EmbedVideoBuilder {
    pub fn new(url: impl Into<String>) -> Self {
        Self(EmbedVideo {
            height: None,
            proxy_url: None,
            url: Some(url.into()),
            width: None,
        })
    }

    pub fn height(mut self, height: impl Into<u64>) -> Self {
        self.0.height = Some(height.into());
        self
    }

    pub fn width(mut self, width: impl Into<u64>) -> Self {
        self.0.width = Some(width.into());
        self
    }

    pub fn build(self) -> EmbedVideo {
        self.0
    }

    pub fn get(&self) -> &EmbedVideo {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut EmbedVideo {
        &mut self.0
    }
}

impl Default for EmbedVideoBuilder {
    fn default() -> Self {
        Self::new("".to_owned())
    }
}

impl From<EmbedVideoBuilder> for EmbedVideo {
    fn from(builder: EmbedVideoBuilder) -> Self {
        builder.build()
    }
}
