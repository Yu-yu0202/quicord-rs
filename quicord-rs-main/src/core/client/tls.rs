/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

/// One-time initializer guard for rustls.
static INIT_RUSTLS: std::sync::Once = std::sync::Once::new();

/// Installs the default rustls crypto provider once per process.
pub(crate) fn init_rustls() -> () {
    INIT_RUSTLS.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}
