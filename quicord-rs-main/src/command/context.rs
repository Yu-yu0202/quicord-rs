/*
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use crate::command::{scope::CommandScope, CommandHandler};

pub struct UserContextCommandMetadata {
    pub name: &'static str,
    pub scope: CommandScope,
    pub run: CommandHandler,
}

#[linkme::distributed_slice]
pub static USER_CONTEXT_COMMANDS: [UserContextCommandMetadata];

pub struct MessageContextCommandMetadata {
    pub name: &'static str,
    pub scope: CommandScope,
    pub run: CommandHandler,
}

#[linkme::distributed_slice]
pub static MESSAGE_CONTEXT_COMMANDS: [MessageContextCommandMetadata];
