/*
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

#[allow(unused_imports)]
pub use quicord_rs_main::*;

pub use linkme;

#[allow(unused_imports)]
#[cfg(feature = "macros")]
pub mod macros {
    pub use quicord_rs_macros::*;
}
