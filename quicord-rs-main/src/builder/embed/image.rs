/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use twilight_model::channel::message::embed::EmbedImage;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct EmbedImageBuilder(EmbedImage);

impl EmbedImageBuilder {
    pub fn new(url: impl Into<String>) -> Self {
        Self(EmbedImage {
            height: None,
            proxy_url: None,
            url: url.into(),
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

    pub fn build(self) -> EmbedImage {
        self.0
    }

    pub fn get(&self) -> &EmbedImage {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut EmbedImage {
        &mut self.0
    }
}

impl Default for EmbedImageBuilder {
    fn default() -> Self {
        Self::new("".to_owned())
    }
}

impl From<EmbedImageBuilder> for EmbedImage {
    fn from(builder: EmbedImageBuilder) -> Self {
        builder.0
    }
}
