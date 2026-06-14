/*
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use proc_macro2::Span;
use syn::{
    Error, Expr, ExprCall, ExprLit, ExprPath, Lit, LitStr, Meta, Path, Result, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

/// Parsed arguments accepted by the command attribute macros.
pub(crate) struct CommandArgs {
    pub(crate) name: Option<LitStr>,
    pub(crate) description: Option<LitStr>,
    pub(crate) scope: Option<ScopeArg>,
    pub(crate) options: Option<Vec<CommandOptionSpec>>,
}

/// Command registration scope parsed from the attribute input.
pub(crate) enum ScopeArg {
    /// Register globally.
    Global,
    /// Register for the listed guild IDs.
    Guild(Vec<LitStr>),
}

/// Parsed metadata for a single slash command option.
pub(crate) struct CommandOptionSpec {
    pub(crate) kind: CommandOptionKind,
    pub(crate) name: LitStr,
    pub(crate) description: LitStr,
    pub(crate) required: bool,
}

/// Supported slash command option kinds.
pub(crate) enum CommandOptionKind {
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
    /// Parses `key = value` pairs from the attribute input.
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

/// Emits a duplicate-attribute error when the field is already set.
fn reject_duplicate<T>(current: &Option<T>, span: Span, key: &str) -> Result<()> {
    if current.is_some() {
        Err(Error::new(span, format!("duplicate `{key}` attribute")))
    } else {
        Ok(())
    }
}

/// Parses a string literal expression.
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

/// Parses the `scope` attribute value.
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

/// Parses an array of command option specs.
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

/// Parses a single command option spec constructor.
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

    if let Some(arg) = arg_iter.next() {
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

/// Converts an identifier path into a supported option kind.
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

/// Parses a boolean literal expression.
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

/// Returns whether the path is exactly the provided identifier.
fn path_is_ident(path: &Path, ident: &str) -> bool {
    path.leading_colon.is_none() && path.segments.len() == 1 && path.segments[0].ident == ident
}
