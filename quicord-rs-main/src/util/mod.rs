/*
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

//! Internal utility helpers used by the runtime.

/// Logging initialization helpers.
pub mod logger;
/// Static routing table for distributed slice metadata.
pub(crate) mod static_router;
