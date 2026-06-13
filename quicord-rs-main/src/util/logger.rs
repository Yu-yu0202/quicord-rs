/*
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init_logger() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .with_ansi(true)
        .with_target(false)
        .with_thread_ids(false);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}
