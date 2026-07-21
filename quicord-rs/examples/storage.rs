// SPDX-License-Identifier: CC0-1.0

use std::sync::atomic::{AtomicU64, Ordering};

use quicord_rs::macros::{event, slash_command};
use quicord_rs::{
    BotBuilder, EventContext, InteractionContext, builder::response::ResponseBuilder,
};

/// Shared data registered once at startup and read from handlers.
struct AppState {
    bot_name: String,
    greeting: String,
    requests: AtomicU64,
}

#[event(event = "ready", once = true)]
async fn on_ready(ctx: EventContext) -> anyhow::Result<()> {
    let state = ctx.storage::<AppState>()?;
    quicord_rs::log::info!(
        "Bot is ready as {} (greeting: {})",
        state.bot_name,
        state.greeting
    );
    Ok(())
}

#[slash_command(name = "greet", description = "Greets you using shared bot data!", scope = global)]
async fn greet(ctx: InteractionContext) -> anyhow::Result<()> {
    let state = ctx.storage::<AppState>()?;
    let count = state.requests.fetch_add(1, Ordering::Relaxed) + 1;

    let name = ctx
        .author()
        .map(|user| user.name.as_str())
        .unwrap_or("stranger");

    let res = ResponseBuilder::new()
        .content(format!(
            "{}, {}! (request #{count} handled by {})",
            state.greeting, name, state.bot_name
        ))
        .build();

    ctx.reply(res).await?;
    Ok(())
}

#[slash_command(
    name = "bot_info",
    description = "Shows data shared across handlers!",
    scope = global
)]
async fn bot_info(ctx: InteractionContext) -> anyhow::Result<()> {
    let state = ctx.storage::<AppState>()?;
    let requests = state.requests.load(Ordering::Relaxed);

    let res = ResponseBuilder::new()
        .content(format!(
            "Bot name: {}\nGreeting: {}\nHandled /greet requests: {requests}",
            state.bot_name, state.greeting
        ))
        .build();

    ctx.reply(res).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bot = BotBuilder::new(std::env::var("DISCORD_TOKEN")?)
        .storage()
        .insert::<AppState>(AppState {
            bot_name: "quicord-storage-example".into(),
            greeting: "Hello".into(),
            requests: AtomicU64::new(0),
        })
        .build()
        .await?;

    bot.start().await?;

    Ok(())
}
