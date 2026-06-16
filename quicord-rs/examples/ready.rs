// SPDX-License-Identifier: CC0-1.0

use quicord_rs::core::event::EventContext;
use quicord_rs::macros::event;
use quicord_rs_main::BotBuilder;

#[event(event = "ready", once = true)]
async fn on_ready(_ctx: EventContext) -> anyhow::Result<()> {
    println!("Bot is ready!",);
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bot = BotBuilder::new(std::env::var("DISCORD_TOKEN")?)
        .build()
        .await?;
    bot.start().await?;

    Ok(())
}
