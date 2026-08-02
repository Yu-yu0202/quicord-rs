/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use crate::builder::button::ButtonBuilder;
use twilight_model::channel::message::component::{Button, ButtonStyle};

pub trait IntoButton {
    fn into_button(self) -> Button;
}

impl IntoButton for Button {
    fn into_button(self) -> Button {
        self
    }
}

impl IntoButton for ButtonBuilder {
    fn into_button(self) -> Button {
        self.build()
    }
}

impl<T, R> IntoButton for T
where
    T: FnOnce(ButtonBuilder) -> R,
    R: Into<Button>,
{
    fn into_button(self) -> Button {
        self(ButtonBuilder::new(ButtonStyle::Primary)).into()
    }
}
