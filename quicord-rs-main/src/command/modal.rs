/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use crate::command::CommandHandler;

/// Metadata describing a registered modal.
pub struct ModalMetadata {
    /// The modal custom ID in Discord.
    pub custom_id: &'static str,
    /// The handler invoked when the modal is submitted.
    pub run: CommandHandler,
}

/// Distributed slice of all registered modals.
#[linkme::distributed_slice]
pub static MODALS: [ModalMetadata];
