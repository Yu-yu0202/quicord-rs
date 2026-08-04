/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use quote::{format_ident, quote};
use syn::{Error, FnArg, ItemFn, Result, Type};

use crate::args::{CommandOptionKind, CommandOptionSpec, ScopeArg};
use crate::definition::{
    CommandDefinition, CommandKind, ComponentDefinition, ContextKind, EventDefinition,
    MessageComponentsKind,
};

/// Validates the input and dispatches to the correct expansion routine.
pub(crate) fn command(
    args: crate::args::CommandArgs,
    item_fn: ItemFn,
    kind: CommandKind,
) -> proc_macro2::TokenStream {
    if let Err(error) = validate_handler(&item_fn, "InteractionContext", "command") {
        return error.to_compile_error();
    }

    let definition = match CommandDefinition::parse(args, kind) {
        Ok(definition) => definition,
        Err(error) => return error.to_compile_error(),
    };

    match definition {
        CommandDefinition::Slash {
            name,
            description,
            scope,
            options,
        } => slash_command(name, description, scope, options, item_fn),
        CommandDefinition::Context { name, scope, kind } => {
            context_command(name, scope, kind, item_fn)
        }
    }
}

/// Validates the input and dispatches to the event handler expansion routine.
pub(crate) fn event(args: crate::args::EventArgs, item_fn: ItemFn) -> proc_macro2::TokenStream {
    if let Err(error) = validate_handler(&item_fn, "EventContext", "event") {
        return error.to_compile_error();
    }

    let EventDefinition { event_type, once } = match EventDefinition::parse(args) {
        Ok(definition) => definition,
        Err(error) => return error.to_compile_error(),
    };

    let event_type_upper = event_type.value().to_uppercase();
    let event_type = syn::LitStr::new(&event_type_upper, event_type.span());

    let handler_name = item_fn.sig.ident.clone();
    let handler_fn = format_ident!("__quicord_rs_{}_event_handler", handler_name);
    let metadata = format_ident!("__quicord_rs_{}_event_metadata", handler_name);

    quote! {
        #item_fn

        fn #handler_fn(
            ctx: ::quicord_rs::core::event::EventContext
        ) -> ::quicord_rs::core::event::EventFuture {
            ::std::boxed::Box::pin(#handler_name(ctx))
        }

        #[quicord_rs::linkme::distributed_slice(::quicord_rs::core::event::EVENT_HANDLERS)]
        #[linkme(crate = ::quicord_rs::linkme)]
        #[allow(non_upper_case_globals)]
        static #metadata: ::quicord_rs::core::event::EventHandlerMetadata = ::quicord_rs::core::event::EventHandlerMetadata {
            event_type: #event_type,
            handler: #handler_fn,
            once: #once,
        };
    }
}

/// Validates the input and dispatches to the message components handler expansion routine.
pub(crate) fn message_components(
    args: crate::args::MessageComponentsArgs,
    item_fn: ItemFn,
    kind: MessageComponentsKind,
) -> proc_macro2::TokenStream {
    if let Err(error) = validate_handler(&item_fn, "InteractionContext", "component") {
        return error.to_compile_error();
    }

    let definition = match ComponentDefinition::parse(args, kind) {
        Ok(definition) => definition,
        Err(error) => return error.to_compile_error(),
    };
    let ComponentDefinition { custom_id, kind } = definition;

    match kind {
        MessageComponentsKind::Button => button(custom_id, item_fn),
        MessageComponentsKind::SelectMenu => select_menu(custom_id, item_fn),
        MessageComponentsKind::Modal => modal(custom_id, item_fn),
    }
}

/// Expands a slash command handler into a handler function and metadata entry.
fn slash_command(
    name: syn::LitStr,
    description: syn::LitStr,
    scope: ScopeArg,
    options: Vec<CommandOptionSpec>,
    item_fn: ItemFn,
) -> proc_macro2::TokenStream {
    let options_tokens = option_tokens(options);

    let command_fn = &item_fn.sig.ident;
    let handler_fn = format_ident!("__quicord_rs_{}_slash_handler", command_fn);
    let metadata = format_ident!("__quicord_rs_{}_slash_metadata", command_fn);
    let scope = scope_tokens(scope);

    quote! {
        #item_fn

        fn #handler_fn(
            ctx: ::quicord_rs::core::interaction::InteractionContext,
        ) -> ::quicord_rs::command::CommandFuture {
            ::std::boxed::Box::pin(#command_fn(ctx))
        }

        #[quicord_rs::linkme::distributed_slice(::quicord_rs::command::slash::SLASH_COMMANDS)]
        #[linkme(crate = ::quicord_rs::linkme)]
        #[allow(non_upper_case_globals)]
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

/// Expands a context command handler into a handler function and metadata entry.
fn context_command(
    name: syn::LitStr,
    scope: ScopeArg,
    kind: ContextKind,
    item_fn: ItemFn,
) -> proc_macro2::TokenStream {
    let command_fn = &item_fn.sig.ident;
    let scope = scope_tokens(scope);

    match kind {
        ContextKind::Message => {
            let handler_fn = format_ident!("__quicord_rs_{}_message_context_handler", command_fn);
            let metadata = format_ident!("__quicord_rs_{}_message_context_metadata", command_fn);

            quote! {
                #item_fn

                fn #handler_fn(
                    ctx: ::quicord_rs::core::interaction::InteractionContext,
                ) -> ::quicord_rs::command::CommandFuture {
                    ::std::boxed::Box::pin(#command_fn(ctx))
                }

                #[quicord_rs::linkme::distributed_slice(::quicord_rs::command::context::MESSAGE_CONTEXT_COMMANDS)]
                #[linkme(crate = ::quicord_rs::linkme)]
                #[allow(non_upper_case_globals)]
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
            let metadata = format_ident!("__quicord_rs_{}_user_context_metadata", command_fn);

            quote! {
                #item_fn

                fn #handler_fn(
                    ctx: ::quicord_rs::core::interaction::InteractionContext,
                ) -> ::quicord_rs::command::CommandFuture {
                    ::std::boxed::Box::pin(#command_fn(ctx))
                }

                #[quicord_rs::linkme::distributed_slice(::quicord_rs::command::context::USER_CONTEXT_COMMANDS)]
                #[linkme(crate = ::quicord_rs::linkme)]
                #[allow(non_upper_case_globals)]
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

fn button(custom_id: syn::LitStr, item_fn: ItemFn) -> proc_macro2::TokenStream {
    let handler_name = item_fn.sig.ident.clone();
    let handler_fn = format_ident!("__quicord_rs_{}_button_handler", handler_name);
    let metadata = format_ident!("__quicord_rs_{}_button_metadata", handler_name);

    quote! {
        #item_fn

        fn #handler_fn(
            ctx: ::quicord_rs::core::interaction::InteractionContext,
        ) -> ::quicord_rs::command::CommandFuture {
            ::std::boxed::Box::pin(#handler_name(ctx))
        }

        #[quicord_rs::linkme::distributed_slice(::quicord_rs::command::message_component::BUTTONS)]
        #[linkme(crate = ::quicord_rs::linkme)]
        #[allow(non_upper_case_globals)]
        static #metadata: ::quicord_rs::command::message_component::ButtonMetadata =
            ::quicord_rs::command::message_component::ButtonMetadata {
                custom_id: #custom_id,
                run: #handler_fn,
            };
    }
}

fn select_menu(custom_id: syn::LitStr, item_fn: ItemFn) -> proc_macro2::TokenStream {
    let handler_name = item_fn.sig.ident.clone();
    let handler_fn = format_ident!("__quicord_rs_{}_select_menu_handler", handler_name);
    let metadata = format_ident!("__quicord_rs_{}_select_menu_metadata", handler_name);

    quote! {
        #item_fn

        fn #handler_fn(
            ctx: ::quicord_rs::core::interaction::InteractionContext,
        ) -> ::quicord_rs::command::CommandFuture {
            ::std::boxed::Box::pin(#handler_name(ctx))
        }

        #[quicord_rs::linkme::distributed_slice(::quicord_rs::command::message_component::SELECT_MENUS)]
        #[linkme(crate = ::quicord_rs::linkme)]
        #[allow(non_upper_case_globals)]
        static #metadata: ::quicord_rs::command::message_component::SelectMenuMetadata =
            ::quicord_rs::command::message_component::SelectMenuMetadata {
                custom_id: #custom_id,
                run: #handler_fn,
            };
    }
}

fn modal(custom_id: syn::LitStr, item_fn: ItemFn) -> proc_macro2::TokenStream {
    let handler_name = item_fn.sig.ident.clone();
    let handler_fn = format_ident!("__quicord_rs_{}_modal_handler", handler_name);
    let metadata = format_ident!("__quicord_rs_{}_modal_metadata", handler_name);

    quote! {
        #item_fn

        fn #handler_fn(
            ctx: ::quicord_rs::core::interaction::InteractionContext,
        ) -> ::quicord_rs::command::CommandFuture {
            ::std::boxed::Box::pin(#handler_name(ctx))
        }

        #[quicord_rs::linkme::distributed_slice(::quicord_rs::command::modal::MODALS)]
        #[linkme(crate = ::quicord_rs::linkme)]
        #[allow(non_upper_case_globals)]
        static #metadata: ::quicord_rs::command::modal::ModalMetadata =
            ::quicord_rs::command::modal::ModalMetadata {
                custom_id: #custom_id,
                run: #handler_fn,
            };
    }
}

/// Checks the handler shape before generating a wrapper that invokes it.
fn validate_handler(item_fn: &ItemFn, context_name: &str, handler_kind: &str) -> Result<()> {
    if item_fn.sig.asyncness.is_none() {
        return Err(Error::new_spanned(
            item_fn.sig.fn_token,
            format!("{handler_kind} handler must be async"),
        ));
    }

    if item_fn.sig.inputs.len() != 1 {
        return Err(Error::new_spanned(
            &item_fn.sig.inputs,
            format!("{handler_kind} handler must accept exactly one {context_name} argument"),
        ));
    }

    let Some(FnArg::Typed(argument)) = item_fn.sig.inputs.first() else {
        return Err(Error::new_spanned(
            &item_fn.sig.inputs,
            format!("{handler_kind} handler must accept a {context_name} argument"),
        ));
    };

    let Type::Path(path) = argument.ty.as_ref() else {
        return Err(Error::new_spanned(
            &argument.ty,
            format!("{handler_kind} handler argument must be {context_name}"),
        ));
    };

    if path.qself.is_some()
        || path
            .path
            .segments
            .last()
            .map(|segment| segment.ident != context_name)
            != Some(false)
    {
        return Err(Error::new_spanned(
            &argument.ty,
            format!("{handler_kind} handler argument must be {context_name}"),
        ));
    }

    Ok(())
}

/// Converts parsed scope information into generated tokens.
fn scope_tokens(scope: ScopeArg) -> proc_macro2::TokenStream {
    match scope {
        ScopeArg::Global => quote! {
            ::quicord_rs::command::scope::CommandScope::Global
        },
        ScopeArg::Guild(guild_ids) => quote! {
            ::quicord_rs::command::scope::CommandScope::Guild(&[
                #(::quicord_rs::twilight_model::id::Id::new(#guild_ids)),*
            ])
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
            ::quicord_rs::command::slash::SlashCommandOptionMetadata {
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
            quote!(::quicord_rs::command::slash::CommandOptionType::Attachment)
        }
        CommandOptionKind::Boolean => {
            quote!(::quicord_rs::command::slash::CommandOptionType::Boolean)
        }
        CommandOptionKind::Channel => {
            quote!(::quicord_rs::command::slash::CommandOptionType::Channel)
        }
        CommandOptionKind::Integer => {
            quote!(::quicord_rs::command::slash::CommandOptionType::Integer)
        }
        CommandOptionKind::Mentionable => {
            quote!(::quicord_rs::command::slash::CommandOptionType::Mentionable)
        }
        CommandOptionKind::Number => {
            quote!(::quicord_rs::command::slash::CommandOptionType::Number)
        }
        CommandOptionKind::Role => {
            quote!(::quicord_rs::command::slash::CommandOptionType::Role)
        }
        CommandOptionKind::String => {
            quote!(::quicord_rs::command::slash::CommandOptionType::String)
        }
        CommandOptionKind::User => {
            quote!(::quicord_rs::command::slash::CommandOptionType::User)
        }
    }
}
