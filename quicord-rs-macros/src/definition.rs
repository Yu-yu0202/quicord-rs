/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0.
 */

use proc_macro2::Span;
use syn::{Error, LitStr, Result};

use crate::args::{CommandArgs, EventArgs, MessageComponentsArgs, ScopeArg};

/// Kind of command being generated.
pub(crate) enum CommandKind {
    Slash,
    MessageContext,
    UserContext,
}

/// Kind of interaction component being generated.
pub(crate) enum MessageComponentsKind {
    Button,
    SelectMenu,
    Modal,
}

/// Validated command definition passed to code generation.
pub(crate) enum CommandDefinition {
    Slash {
        name: LitStr,
        description: LitStr,
        scope: ScopeArg,
        options: Vec<crate::args::CommandOptionSpec>,
    },
    Context {
        name: LitStr,
        scope: ScopeArg,
        kind: ContextKind,
    },
}

/// Context command kind in a validated definition.
pub(crate) enum ContextKind {
    Message,
    User,
}

/// Validated event definition passed to code generation.
pub(crate) struct EventDefinition {
    pub(crate) event_type: LitStr,
    pub(crate) once: bool,
}

/// Validated component definition passed to code generation.
pub(crate) struct ComponentDefinition {
    pub(crate) custom_id: LitStr,
    pub(crate) kind: MessageComponentsKind,
}

impl CommandDefinition {
    pub(crate) fn parse(args: CommandArgs, kind: CommandKind) -> Result<Self> {
        let CommandArgs {
            name,
            description,
            scope,
            options,
        } = args;

        let name = required(name, "name")?;
        let scope = required(scope, "scope")?;

        match kind {
            CommandKind::Slash => Ok(Self::Slash {
                name,
                description: required(description, "description")?,
                scope,
                options: options.unwrap_or_default(),
            }),
            CommandKind::MessageContext | CommandKind::UserContext => {
                if let Some(description) = description {
                    return Err(Error::new_spanned(
                        description,
                        "context commands cannot have a description in the Discord API",
                    ));
                }
                if let Some(options) = options {
                    let first = options
                        .into_iter()
                        .next()
                        .expect("parsed options cannot be empty when present");
                    return Err(Error::new_spanned(
                        first.name,
                        "context commands cannot declare options",
                    ));
                }

                let kind = match kind {
                    CommandKind::MessageContext => ContextKind::Message,
                    CommandKind::UserContext => ContextKind::User,
                    CommandKind::Slash => unreachable!(),
                };

                Ok(Self::Context { name, scope, kind })
            }
        }
    }
}

impl EventDefinition {
    pub(crate) fn parse(args: EventArgs) -> Result<Self> {
        Ok(Self {
            event_type: required(args.event_type, "event")?,
            once: args.once.unwrap_or(false),
        })
    }
}

impl ComponentDefinition {
    pub(crate) fn parse(args: MessageComponentsArgs, kind: MessageComponentsKind) -> Result<Self> {
        Ok(Self {
            custom_id: required(args.custom_id, "custom_id")?,
            kind,
        })
    }
}

fn required<T>(value: Option<T>, key: &str) -> Result<T> {
    value.ok_or_else(|| Error::new(Span::call_site(), format!("missing `{key}` attribute")))
}
