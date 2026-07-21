/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use twilight_model::channel::message::Component;
use twilight_model::channel::message::component::{ActionRow, SelectMenu, TextInput};
use twilight_model::http::interaction::InteractionResponseData;

pub struct ModalBuilder(InteractionResponseData);

impl ModalBuilder {
    pub fn new(custom_id: impl Into<String>, title: impl Into<String>) -> Self {
        Self(InteractionResponseData {
            custom_id: Some(custom_id.into()),
            title: Some(title.into()),
            components: Some(Vec::new()),
            ..Default::default()
        })
    }

    pub fn text_input(mut self, component: impl Into<TextInput>) -> Self {
        let components = self.0.components.get_or_insert_with(Vec::new);
        components.push(into_action_row_component(component.into()));
        self
    }

    pub fn select_menu(mut self, component: impl Into<SelectMenu>) -> Self {
        let components = self.0.components.get_or_insert_with(Vec::new);
        components.push(into_action_row_component(component.into()));
        self
    }

    pub fn action_row(mut self, component: impl Into<ActionRow>) -> Self {
        let components = self.0.components.get_or_insert_with(Vec::new);
        components.push(component.into().into());
        self
    }

    pub fn component(mut self, component: impl Into<Component>) -> Self {
        let components = self.0.components.get_or_insert_with(Vec::new);
        components.push(component.into());
        self
    }

    pub fn components(
        mut self,
        components: impl IntoIterator<Item = impl Into<Component>>,
    ) -> Self {
        let components_vec = self.0.components.get_or_insert_with(Vec::new);
        components_vec.extend(components.into_iter().map(|c| c.into()));
        self
    }

    pub fn get(&self) -> &InteractionResponseData {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut InteractionResponseData {
        &mut self.0
    }

    pub fn clear_components(mut self) -> Self {
        self.0.components = Some(Vec::new());
        self
    }

    pub fn build(self) -> InteractionResponseData {
        self.0
    }
}

impl From<ModalBuilder> for InteractionResponseData {
    fn from(builder: ModalBuilder) -> Self {
        builder.build()
    }
}

fn into_action_row_component(component: impl Into<Component>) -> Component {
    Component::ActionRow(ActionRow {
        id: None,
        components: vec![component.into()],
    })
}
