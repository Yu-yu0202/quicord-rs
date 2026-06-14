/*
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{Error, ItemFn, Result};

use crate::args::{CommandArgs, CommandOptionKind, CommandOptionSpec, ScopeArg};

/// Discriminant used while expanding a command handler.
pub(crate) enum CommandKind {
    /// Slash command expansion.
    Slash,
    /// Message context command expansion.
    MessageContext,
    /// User context command expansion.
    UserContext,
}

/// Validates the input and dispatches to the correct expansion routine.
pub(crate) fn command(
    args: CommandArgs,
    item_fn: ItemFn,
    kind: CommandKind,
) -> proc_macro2::TokenStream {
    if item_fn.sig.asyncness.is_none() {
        return Error::new_spanned(item_fn.sig.fn_token, "command handler must be async")
            .to_compile_error();
    }

    match kind {
        CommandKind::Slash => slash_command(args, item_fn),
        CommandKind::MessageContext => context_command(args, item_fn, ContextKind::Message),
        CommandKind::UserContext => context_command(args, item_fn, ContextKind::User),
    }
}

/// Expands a slash command handler into a handler function and metadata entry.
fn slash_command(args: CommandArgs, item_fn: ItemFn) -> proc_macro2::TokenStream {
    let (name, description, scope) = match (
        required(args.name, "name", Span::call_site()),
        required(args.description, "description", Span::call_site()),
        required(args.scope, "scope", Span::call_site()),
    ) {
        (Ok(name), Ok(description), Ok(scope)) => (name, description, scope),
        (name, description, scope) => {
            return combine_errors([name.err(), description.err(), scope.err()]);
        }
    };
    let options = args.options.unwrap_or_default();
    let options_tokens = option_tokens(options);

    let command_fn = &item_fn.sig.ident;
    let handler_fn = format_ident!("__quicord_rs_{}_slash_handler", command_fn);
    let metadata = format_ident!(
        "__QUICORD_RS_{}_SLASH_COMMAND",
        command_fn.to_string().to_uppercase()
    );
    let scope = scope_tokens(scope);

    quote! {
        #item_fn

        fn #handler_fn(
            ctx: ::quicord_rs::core::interaction::InteractionContext,
        ) -> ::quicord_rs::command::CommandFuture {
            ::std::boxed::Box::pin(#command_fn(ctx))
        }

        #[quicord_rs::linkme::distributed_slice(::quicord_rs::command::slash::SLASH_COMMANDS)]
        static #metadata: ::quicord_rs::command::slash::SlashCommandMetadata =
            ::quicord_rs::command::slash::SlashCommandMetadata {
                name: #name,
                description: #description,
                scope: #scope,
                options: #options_tokens,
                run: #handler_fn,
            };
    }
}

/// Distinguishes the two supported context command kinds.
enum ContextKind {
    /// Message context command.
    Message,
    /// User context command.
    User,
}

/// Expands a context command handler into a handler function and metadata entry.
fn context_command(
    args: CommandArgs,
    item_fn: ItemFn,
    kind: ContextKind,
) -> proc_macro2::TokenStream {
    if let Some(description) = args.description {
        return Error::new_spanned(
            description,
            "context commands cannot have a description in the Discord API",
        )
        .to_compile_error();
    }
    if let Some(options) = args.options {
        let first = options.into_iter().next().unwrap();
        return Error::new_spanned(first.name, "context commands cannot declare options")
            .to_compile_error();
    }

    let (name, scope) = match (
        required(args.name, "name", Span::call_site()),
        required(args.scope, "scope", Span::call_site()),
    ) {
        (Ok(name), Ok(scope)) => (name, scope),
        (name, scope) => {
            return combine_errors([name.err(), scope.err()]);
        }
    };

    let command_fn = &item_fn.sig.ident;
    let scope = scope_tokens(scope);

    match kind {
        ContextKind::Message => {
            let handler_fn = format_ident!("__quicord_rs_{}_message_context_handler", command_fn);
            let metadata = format_ident!(
                "__QUICORD_RS_{}_MESSAGE_CONTEXT_COMMAND",
                command_fn.to_string().to_uppercase()
            );

            quote! {
                #item_fn

                fn #handler_fn(
                    ctx: ::quicord_rs::core::interaction::InteractionContext,
                ) -> ::quicord_rs::command::CommandFuture {
                    ::std::boxed::Box::pin(#command_fn(ctx))
                }

                #[quicord_rs::linkme::distributed_slice(::quicord_rs::command::context::MESSAGE_CONTEXT_COMMANDS)]
                static #metadata: ::quicord_rs::command::context::MessageContextCommandMetadata =
                    ::quicord_rs::command::context::MessageContextCommandMetadata {
                        name: #name,
                        scope: #scope,
                        run: #handler_fn,
                    };
            }
        }
        ContextKind::User => {
            let handler_fn = format_ident!("__quicord_rs_{}_user_context_handler", command_fn);
            let metadata = format_ident!(
                "__QUICORD_RS_{}_USER_CONTEXT_COMMAND",
                command_fn.to_string().to_uppercase()
            );

            quote! {
                #item_fn

                fn #handler_fn(
                    ctx: ::quicord_rs::core::interaction::InteractionContext,
                ) -> ::quicord_rs::command::CommandFuture {
                    ::std::boxed::Box::pin(#command_fn(ctx))
                }

                #[quicord_rs::linkme::distributed_slice(::quicord_rs::command::context::USER_CONTEXT_COMMANDS)]
                static #metadata: ::quicord_rs::command::context::UserContextCommandMetadata =
                    ::quicord_rs::command::context::UserContextCommandMetadata {
                        name: #name,
                        scope: #scope,
                        run: #handler_fn,
                    };
            }
        }
    }
}

/// Emits a missing-attribute error for a required field.
fn required<T>(value: Option<T>, key: &str, span: Span) -> Result<T> {
    value.ok_or_else(|| Error::new(span, format!("missing `{key}` attribute")))
}

/// Converts parsed scope information into generated tokens.
fn scope_tokens(scope: ScopeArg) -> proc_macro2::TokenStream {
    match scope {
        ScopeArg::Global => quote! {
            ::quicord_rs_main::command::scope::CommandScope::Global
        },
        ScopeArg::Guild(guild_ids) => quote! {
            ::quicord_rs_main::command::scope::CommandScope::Guild(&[#(#guild_ids),*])
        },
    }
}

/// Converts parsed slash option metadata into generated tokens.
fn option_tokens(options: Vec<CommandOptionSpec>) -> proc_macro2::TokenStream {
    let options = options.into_iter().map(|option| {
        let kind = option_kind_tokens(option.kind);
        let name = option.name;
        let description = option.description;
        let required = option.required;

        quote! {
            ::quicord_rs_main::command::slash::SlashCommandOptionMetadata {
                name: #name,
                description: #description,
                kind: #kind,
                required: #required,
            }
        }
    });

    quote! {
        &[
            #(#options),*
        ]
    }
}

/// Converts a parsed option kind into generated tokens.
fn option_kind_tokens(kind: CommandOptionKind) -> proc_macro2::TokenStream {
    match kind {
        CommandOptionKind::Attachment => {
            quote!(::quicord_rs_main::command::slash::CommandOptionType::Attachment)
        }
        CommandOptionKind::Boolean => {
            quote!(::quicord_rs_main::command::slash::CommandOptionType::Boolean)
        }
        CommandOptionKind::Channel => {
            quote!(::quicord_rs_main::command::slash::CommandOptionType::Channel)
        }
        CommandOptionKind::Integer => {
            quote!(::quicord_rs_main::command::slash::CommandOptionType::Integer)
        }
        CommandOptionKind::Mentionable => {
            quote!(::quicord_rs_main::command::slash::CommandOptionType::Mentionable)
        }
        CommandOptionKind::Number => {
            quote!(::quicord_rs_main::command::slash::CommandOptionType::Number)
        }
        CommandOptionKind::Role => {
            quote!(::quicord_rs_main::command::slash::CommandOptionType::Role)
        }
        CommandOptionKind::String => {
            quote!(::quicord_rs_main::command::slash::CommandOptionType::String)
        }
        CommandOptionKind::User => {
            quote!(::quicord_rs_main::command::slash::CommandOptionType::User)
        }
    }
}

/// Combines multiple parser errors into a single compile error.
fn combine_errors(errors: impl IntoIterator<Item = Option<Error>>) -> proc_macro2::TokenStream {
    let mut combined: Option<Error> = None;

    for error in errors.into_iter().flatten() {
        if let Some(combined) = &mut combined {
            combined.combine(error);
        } else {
            combined = Some(error);
        }
    }

    combined
        .map(|error| error.to_compile_error())
        .unwrap_or_default()
}
