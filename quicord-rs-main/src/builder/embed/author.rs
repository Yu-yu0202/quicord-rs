/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
use twilight_model::channel::message::embed::EmbedAuthor;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct EmbedAuthorBuilder(EmbedAuthor);

impl EmbedAuthorBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self(EmbedAuthor {
            icon_url: None,
            name: name.into(),
            proxy_icon_url: None,
            url: None,
        })
    }

    pub fn icon_url(mut self, url: impl Into<String>) -> Self {
        self.0.icon_url = Some(url.into());
        self
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.0.url = Some(url.into());
        self
    }

    pub fn build(self) -> EmbedAuthor {
        self.0
    }

    pub fn get(&self) -> &EmbedAuthor {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut EmbedAuthor {
        &mut self.0
    }
}

impl From<EmbedAuthorBuilder> for EmbedAuthor {
    fn from(builder: EmbedAuthorBuilder) -> Self {
        builder.build()
    }
}
