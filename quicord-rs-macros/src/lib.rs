/*
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream}, parse_macro_input, punctuated::Punctuated, Error, Expr, ExprCall, ExprLit, ExprPath, ItemFn, Lit, LitStr, Meta,
    Path,
    Result,
    Token,
};

struct CommandArgs {
    name: Option<LitStr>,
    description: Option<LitStr>,
    scope: Option<ScopeArg>,
    options: Option<Vec<CommandOptionSpec>>,
}

enum ScopeArg {
    Global,
    Guild(Vec<LitStr>),
}

struct CommandOptionSpec {
    kind: CommandOptionKind,
    name: LitStr,
    description: LitStr,
    required: bool,
}

enum CommandOptionKind {
    Attachment,
    Boolean,
    Channel,
    Integer,
    Mentionable,
    Number,
    Role,
    String,
    User,
}

impl Parse for CommandArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let args = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        let mut parsed = CommandArgs {
            name: None,
            description: None,
            scope: None,
            options: None,
        };

        for arg in args {
            let Meta::NameValue(name_value) = arg else {
                return Err(Error::new_spanned(arg, "expected `key = value`"));
            };

            let Some(ident) = name_value.path.get_ident() else {
                return Err(Error::new_spanned(name_value.path, "expected simple key"));
            };

            match ident.to_string().as_str() {
                "name" => {
                    reject_duplicate(&parsed.name, ident.span(), "name")?;
                    parsed.name = Some(parse_string_literal(name_value.value, "name")?);
                }
                "description" => {
                    reject_duplicate(&parsed.description, ident.span(), "description")?;
                    parsed.description =
                        Some(parse_string_literal(name_value.value, "description")?);
                }
                "scope" => {
                    reject_duplicate(&parsed.scope, ident.span(), "scope")?;
                    parsed.scope = Some(parse_scope(name_value.value)?);
                }
                "options" => {
                    reject_duplicate(&parsed.options, ident.span(), "options")?;
                    parsed.options = Some(parse_options(name_value.value)?);
                }
                other => {
                    return Err(Error::new_spanned(
                        ident,
                        format!("unknown command attribute `{other}`"),
                    ));
                }
            }
        }

        Ok(parsed)
    }
}

#[proc_macro_attribute]
pub fn slash_command(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_command(
        parse_macro_input!(attr as CommandArgs),
        parse_macro_input!(item as ItemFn),
        CommandKind::Slash,
    )
    .into()
}

#[proc_macro_attribute]
pub fn message_context(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_command(
        parse_macro_input!(attr as CommandArgs),
        parse_macro_input!(item as ItemFn),
        CommandKind::MessageContext,
    )
    .into()
}

#[proc_macro_attribute]
pub fn user_context(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_command(
        parse_macro_input!(attr as CommandArgs),
        parse_macro_input!(item as ItemFn),
        CommandKind::UserContext,
    )
    .into()
}

enum CommandKind {
    Slash,
    MessageContext,
    UserContext,
}

fn expand_command(
    args: CommandArgs,
    item_fn: ItemFn,
    kind: CommandKind,
) -> proc_macro2::TokenStream {
    if item_fn.sig.asyncness.is_none() {
        return Error::new_spanned(item_fn.sig.fn_token, "command handler must be async")
            .to_compile_error();
    }

    match kind {
        CommandKind::Slash => expand_slash_command(args, item_fn),
        CommandKind::MessageContext => expand_context_command(args, item_fn, ContextKind::Message),
        CommandKind::UserContext => expand_context_command(args, item_fn, ContextKind::User),
    }
}

fn expand_slash_command(args: CommandArgs, item_fn: ItemFn) -> proc_macro2::TokenStream {
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

enum ContextKind {
    Message,
    User,
}

fn expand_context_command(
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

fn reject_duplicate<T>(current: &Option<T>, span: Span, key: &str) -> Result<()> {
    if current.is_some() {
        Err(Error::new(span, format!("duplicate `{key}` attribute")))
    } else {
        Ok(())
    }
}

fn required<T>(value: Option<T>, key: &str, span: Span) -> Result<T> {
    value.ok_or_else(|| Error::new(span, format!("missing `{key}` attribute")))
}

fn parse_string_literal(expr: Expr, key: &str) -> Result<LitStr> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) => Ok(value),
        other => Err(Error::new_spanned(
            other,
            format!("`{key}` must be a string literal"),
        )),
    }
}

fn parse_scope(expr: Expr) -> Result<ScopeArg> {
    match expr {
        Expr::Path(ExprPath { path, .. }) if path_is_ident(&path, "global") => Ok(ScopeArg::Global),
        Expr::Call(ExprCall { func, args, .. }) => {
            let Expr::Path(ExprPath { path, .. }) = func.as_ref() else {
                return Err(Error::new_spanned(func, "expected `guild(...)`"));
            };

            if !path_is_ident(path, "guild") {
                return Err(Error::new_spanned(path, "expected `guild(...)`"));
            }

            let guild_ids = args
                .into_iter()
                .map(|arg| parse_string_literal(arg, "guild id"))
                .collect::<Result<Vec<_>>>()?;

            if guild_ids.is_empty() {
                return Err(Error::new_spanned(
                    path,
                    "`guild(...)` requires at least one id",
                ));
            }

            Ok(ScopeArg::Guild(guild_ids))
        }
        other => Err(Error::new_spanned(
            other,
            "scope must be `global` or `guild(\"...\")`",
        )),
    }
}

fn parse_options(expr: Expr) -> Result<Vec<CommandOptionSpec>> {
    match expr {
        Expr::Array(array) => array
            .elems
            .into_iter()
            .map(parse_option_spec)
            .collect::<Result<Vec<_>>>(),
        other => Err(Error::new_spanned(other, "options must be an array")),
    }
}

fn parse_option_spec(expr: Expr) -> Result<CommandOptionSpec> {
    let Expr::Call(ExprCall { func, args, .. }) = expr else {
        return Err(Error::new_spanned(
            expr,
            "expected option constructor like `String(\"name\")`",
        ));
    };

    let Expr::Path(ExprPath { path, .. }) = func.as_ref() else {
        return Err(Error::new_spanned(func, "expected option constructor"));
    };

    let kind = parse_option_kind(path)?;
    let mut arg_iter = args.into_iter();
    let name = match arg_iter.next() {
        Some(arg) => parse_string_literal(arg, "option name")?,
        None => {
            return Err(Error::new_spanned(
                path,
                "option constructor requires at least a name",
            ));
        }
    };
    let mut description = name.clone();
    let mut required = true;

    match arg_iter.next() {
        Some(arg) => {
            if matches!(
                arg,
                Expr::Lit(ExprLit {
                    lit: Lit::Bool(_),
                    ..
                })
            ) {
                required = parse_bool_literal(arg, "option required")?;
            } else {
                description = parse_string_literal(arg, "option description")?;
                if let Some(arg) = arg_iter.next() {
                    required = parse_bool_literal(arg, "option required")?;
                }
            }
        }
        None => {}
    }

    if arg_iter.next().is_some() {
        return Err(Error::new_spanned(
            path,
            "option constructor accepts at most three arguments",
        ));
    }

    Ok(CommandOptionSpec {
        kind,
        name,
        description,
        required,
    })
}

fn parse_option_kind(path: &Path) -> Result<CommandOptionKind> {
    if path_is_ident(path, "Attachment") {
        Ok(CommandOptionKind::Attachment)
    } else if path_is_ident(path, "Boolean") {
        Ok(CommandOptionKind::Boolean)
    } else if path_is_ident(path, "Channel") {
        Ok(CommandOptionKind::Channel)
    } else if path_is_ident(path, "Integer") {
        Ok(CommandOptionKind::Integer)
    } else if path_is_ident(path, "Mentionable") {
        Ok(CommandOptionKind::Mentionable)
    } else if path_is_ident(path, "Number") {
        Ok(CommandOptionKind::Number)
    } else if path_is_ident(path, "Role") {
        Ok(CommandOptionKind::Role)
    } else if path_is_ident(path, "String") {
        Ok(CommandOptionKind::String)
    } else if path_is_ident(path, "User") {
        Ok(CommandOptionKind::User)
    } else {
        Err(Error::new_spanned(
            path,
            "unknown option kind, expected one of Attachment, Boolean, Channel, Integer, Mentionable, Number, Role, String, User",
        ))
    }
}

fn parse_bool_literal(expr: Expr, key: &str) -> Result<bool> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Bool(value),
            ..
        }) => Ok(value.value),
        other => Err(Error::new_spanned(
            other,
            format!("`{key}` must be a boolean literal"),
        )),
    }
}

fn path_is_ident(path: &Path, ident: &str) -> bool {
    path.leading_colon.is_none() && path.segments.len() == 1 && path.segments[0].ident == ident
}

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
