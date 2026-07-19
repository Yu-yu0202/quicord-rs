/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use twilight_model::channel::message::embed::EmbedField;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct EmbedFieldBuilder(EmbedField);

impl EmbedFieldBuilder {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self(EmbedField {
            inline: false,
            name: name.into(),
            value: value.into(),
        })
    }

    pub fn inline(mut self, inline: bool) -> Self {
        self.0.inline = inline;
        self
    }

    pub fn build(self) -> EmbedField {
        self.0
    }

    pub fn get(&self) -> &EmbedField {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut EmbedField {
        &mut self.0
    }
}

impl Default for EmbedFieldBuilder {
    fn default() -> Self {
        Self::new("".to_owned(), "".to_owned())
    }
}

impl From<EmbedFieldBuilder> for EmbedField {
    fn from(builder: EmbedFieldBuilder) -> Self {
        builder.build()
    }
}
