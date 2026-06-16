/*
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

//! Core runtime and command routing types for `quicord-rs`.

extern crate self as quicord_rs_main;

/// Command registration and command handler metadata.
pub mod command;
/// Bot, event loop, and interaction routing primitives.
pub mod core;

/// Reexports of the primary bot types from [`core`].
pub use core::{Bot, BotBuilder, EventContext, InteractionContext};


/// Utility helpers used internally by the runtime.
pub mod util;

/// Reexport of the `linkme` crate used for distributed slices.
pub use linkme;


/// Reexports commonly used tracing macros.
pub mod log {
    pub use tracing::{debug, debug_span, error, error_span, info, info_span, trace, warn};
}
