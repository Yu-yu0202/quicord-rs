/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use crate::builder::text_input::TextInputBuilder;
use twilight_model::channel::message::component::{TextInput, TextInputStyle};

pub trait IntoTextInput {
    fn into_text_input(self) -> TextInput;
}

impl IntoTextInput for TextInput {
    fn into_text_input(self) -> TextInput {
        self
    }
}

impl IntoTextInput for TextInputBuilder {
    fn into_text_input(self) -> TextInput {
        self.build()
    }
}

impl<T, R> IntoTextInput for T
where
    T: FnOnce(TextInputBuilder) -> R,
    R: Into<TextInput>,
{
    fn into_text_input(self) -> TextInput {
        self(TextInputBuilder::new("", TextInputStyle::Short)).into()
    }
}
