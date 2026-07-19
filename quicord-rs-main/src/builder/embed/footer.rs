/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use twilight_model::channel::message::embed::EmbedFooter;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct EmbedFooterBuilder(EmbedFooter);

impl EmbedFooterBuilder {
    pub fn new(text: impl Into<String>) -> Self {
        Self(EmbedFooter {
            icon_url: None,
            proxy_icon_url: None,
            text: text.into(),
        })
    }

    pub fn icon_url(mut self, url: impl Into<String>) -> Self {
        self.0.icon_url = Some(url.into());
        self
    }

    pub fn build(self) -> EmbedFooter {
        self.0
    }

    pub fn get(&self) -> &EmbedFooter {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut EmbedFooter {
        &mut self.0
    }
}

impl Default for EmbedFooterBuilder {
    fn default() -> Self {
        Self::new("".to_owned())
    }
}

impl From<EmbedFooterBuilder> for EmbedFooter {
    fn from(builder: EmbedFooterBuilder) -> Self {
        builder.build()
    }
}
