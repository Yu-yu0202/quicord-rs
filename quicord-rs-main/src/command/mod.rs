pub mod context;
pub mod scope;
pub mod slash;

pub type CommandFuture = futures_util::future::BoxFuture<'static, anyhow::Result<()>>;
pub type CommandHandler = fn(crate::core::interaction::InteractionContext) -> CommandFuture;
