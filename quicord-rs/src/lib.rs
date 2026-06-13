/*
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

//! Public entry point for `quicord-rs`.

#[allow(unused_imports)]
pub use quicord_rs_main::*;

/// Reexport of `linkme` for distributed slice definitions.
pub use linkme;

#[allow(unused_imports)]
#[cfg(feature = "macros")]
/// Reexport of the attribute macros.
pub mod macros {
    pub use quicord_rs_macros::*;
}
