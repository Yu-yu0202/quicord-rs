/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

/// Scope used when registering commands with Discord.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CommandScope {
    /// Register the command globally.
    Global,
    /// Register the command for the listed guild IDs only.
    Guild(&'static [&'static str]),
}
