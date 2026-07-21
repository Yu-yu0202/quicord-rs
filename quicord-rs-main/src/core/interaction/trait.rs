/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use twilight_model::http::interaction::InteractionResponseData;
use twilight_util::builder::InteractionResponseDataBuilder as InteractionResponseBuilder;

/// Converts a value into a Discord interaction response payload.
pub trait IntoResponse {
    /// Builds the response payload.
    fn into_response(self) -> InteractionResponseData;
}

/// Converts a string slice into a plain text response.
impl IntoResponse for &str {
    fn into_response(self) -> InteractionResponseData {
        InteractionResponseData {
            content: Some(self.to_string()),
            ..Default::default()
        }
    }
}

/// Converts an owned string into a plain text response.
impl IntoResponse for String {
    fn into_response(self) -> InteractionResponseData {
        InteractionResponseData {
            content: Some(self),
            ..Default::default()
        }
    }
}

/// Returns an interaction response builder unchanged.
impl IntoResponse for InteractionResponseBuilder {
    fn into_response(self) -> InteractionResponseData {
        self.build()
    }
}

/// Returns an already built interaction response unchanged.
impl IntoResponse for InteractionResponseData {
    fn into_response(self) -> InteractionResponseData {
        self
    }
}
