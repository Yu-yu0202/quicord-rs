/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

//! Command metadata, scopes, and interaction helpers.

/// Metadata for user and message context commands.
pub mod context;
/// Metadata for message component interactions (buttons and select menus).
pub mod message_component;
/// Modal metadata definitions.
pub mod modal;
/// Command visibility and registration scope.
pub mod scope;
/// Slash command metadata and option definitions.
pub mod slash;

/// Future returned by command handlers.
pub type CommandFuture = futures_util::future::BoxFuture<'static, anyhow::Result<()>>;
/// Function signature used by generated command handlers.
pub type CommandHandler = fn(crate::core::interaction::InteractionContext) -> CommandFuture;
