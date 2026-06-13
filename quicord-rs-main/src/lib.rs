/*
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

extern crate self as quicord_rs_main;

pub mod command;
pub mod core;

pub use core::{Bot, BotBuilder, InteractionContext};

pub mod util;

pub use linkme;

pub mod log {
    pub use tracing::{debug, debug_span, error, error_span, info, info_span, trace, warn};
}
