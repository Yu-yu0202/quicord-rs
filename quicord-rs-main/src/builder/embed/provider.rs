/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use twilight_model::channel::message::embed::EmbedProvider;

pub struct EmbedProviderBuilder(EmbedProvider);

impl EmbedProviderBuilder {
    pub fn new() -> Self {
        EmbedProviderBuilder(EmbedProvider {
            name: None,
            url: None,
        })
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.0.name = Some(name.into());
        self
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.0.url = Some(url.into());
        self
    }

    pub fn build(self) -> EmbedProvider {
        self.0
    }

    pub fn get(&self) -> &EmbedProvider {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut EmbedProvider {
        &mut self.0
    }
}

impl Default for EmbedProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<EmbedProviderBuilder> for EmbedProvider {
    fn from(builder: EmbedProviderBuilder) -> Self {
        builder.build()
    }
}
