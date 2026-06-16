/*
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

mod args;
mod expand;

/// Attribute macro that registers a slash command.
#[proc_macro_attribute]
pub fn slash_command(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand::command(
        parse_macro_input!(attr as args::CommandArgs),
        parse_macro_input!(item as ItemFn),
        expand::CommandKind::Slash,
    )
    .into()
}

/// Attribute macro that registers a message context command.
#[proc_macro_attribute]
pub fn message_context(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand::command(
        parse_macro_input!(attr as args::CommandArgs),
        parse_macro_input!(item as ItemFn),
        expand::CommandKind::MessageContext,
    )
    .into()
}

/// Attribute macro that registers a user context command.
#[proc_macro_attribute]
pub fn user_context(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand::command(
        parse_macro_input!(attr as args::CommandArgs),
        parse_macro_input!(item as ItemFn),
        expand::CommandKind::UserContext,
    )
    .into()
}

/// Attribute macro that registers a gateway event handler.
#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand::event(
        parse_macro_input!(attr as args::EventArgs),
        parse_macro_input!(item as ItemFn),
    )
    .into()
}
