/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Attempts to initialize a default global tracing subscriber.
///
/// Applications that already install a subscriber should configure tracing
/// themselves and ignore this helper.
pub fn try_init_logger() -> Result<(), tracing_subscriber::util::TryInitError> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .with_ansi(true)
        .with_target(false)
        .with_thread_ids(false);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init()
}

/// Initializes a default global tracing subscriber when one is not installed.
///
/// Prefer [`try_init_logger`] when initialization failures must be observed.
pub fn init_logger() {
    let _ = try_init_logger();
}
