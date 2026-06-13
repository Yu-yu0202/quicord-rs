/*
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

//! Core runtime components for bot execution and interaction handling.

/// Bot construction, routing, and event loop management.
pub mod client;
/// Gateway event handler metadata.
pub mod event;
/// Interaction context helpers and response conversion traits.
pub mod interaction;

/// Primary bot runtime type.
pub use client::{Bot, BotBuilder};
/// Context passed to interaction handlers.
pub use interaction::InteractionContext;
