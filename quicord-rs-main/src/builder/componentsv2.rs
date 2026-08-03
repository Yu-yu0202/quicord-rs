/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

pub mod r#trait;

use crate::builder::componentsv2::r#trait::button::IntoButton;
use crate::builder::componentsv2::r#trait::text_input::IntoTextInput;
use twilight_model::channel::message::Component;
use twilight_model::channel::message::component::Container;

pub struct ComponentsV2Builder(Container);

impl ComponentsV2Builder {
    pub fn new() -> Self {
        Self(Container {
            id: None,
            accent_color: None,
            components: Vec::new(),
            spoiler: None,
        })
    }

    pub fn id(mut self, id: impl Into<i32>) -> Self {
        self.0.id = Some(id.into());
        self
    }

    pub fn accent_color(mut self, color: impl Into<u32>) -> Self {
        self.0.accent_color = Some(Some(color.into()));
        self
    }

    pub fn spoiler(mut self, spoiler: bool) -> Self {
        self.0.spoiler = Some(spoiler);
        self
    }

    pub fn component(mut self, component: impl Into<Component>) -> Self {
        self.0.components.push(component.into());
        self
    }

    pub fn components(
        mut self,
        components: impl IntoIterator<Item = impl Into<Component>>,
    ) -> Self {
        self.0
            .components
            .extend(components.into_iter().map(Into::into));
        self
    }

    pub fn text_input(mut self, text_input: impl IntoTextInput) -> Self {
        self.0.components.push(text_input.into_text_input().into());
        self
    }

    pub fn text_inputs(
        mut self,
        text_inputs: impl IntoIterator<Item = impl IntoTextInput>,
    ) -> Self {
        self.0
            .components
            .extend(text_inputs.into_iter().map(|x| x.into_text_input().into()));
        self
    }

    pub fn button(mut self, button: impl IntoButton) -> Self {
        self.0.components.push(button.into_button().into());
        self
    }

    pub fn buttons(mut self, buttons: impl IntoIterator<Item = impl IntoButton>) -> Self {
        self.0
            .components
            .extend(buttons.into_iter().map(|x| x.into_button().into()));
        self
    }

    pub fn build(self) -> Container {
        self.0
    }
}

impl Default for ComponentsV2Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Container> for ComponentsV2Builder {
    fn from(container: Container) -> Self {
        Self(container)
    }
}
