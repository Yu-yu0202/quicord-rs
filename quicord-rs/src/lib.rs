/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

//! Public entry point for `quicord-rs`.

extern crate self as quicord_rs;

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

#[cfg(all(test, feature = "macros"))]
mod tests {
    use super::{InteractionContext, macros::slash_command};

    #[slash_command(
        name = "guild_scope_test",
        description = "Verifies guild scope expansion",
        scope = guild(1, "2")
    )]
    async fn guild_scope_test(_: InteractionContext) -> anyhow::Result<()> {
        Ok(())
    }

    #[test]
    fn guild_scope_macro_compiles() {}
}
